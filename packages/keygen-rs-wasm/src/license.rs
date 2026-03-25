use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::license_file::LicenseFile;
use crate::to_js_error;
use crate::token_module::Token;
use crate::user::User;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    pub id: String,
    pub scheme: Option<String>,
    pub key: String,
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub status: Option<String>,
    pub uses: Option<i32>,
    pub max_machines: Option<i32>,
    pub max_cores: Option<i32>,
    pub max_uses: Option<i32>,
    pub max_processes: Option<i32>,
    pub max_users: Option<i32>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub policy: Option<String>,
    pub metadata: serde_json::Value,
    pub account_id: Option<String>,
    pub product_id: Option<String>,
    pub group_id: Option<String>,
    pub owner_id: Option<String>,
}

impl From<keygen_rs::license::License> for License {
    fn from(lic: keygen_rs::license::License) -> Self {
        License {
            id: lic.id,
            scheme: lic
                .scheme
                .as_ref()
                .and_then(|s| serde_json::to_value(s).ok())
                .and_then(|v| v.as_str().map(String::from)),
            key: lic.key,
            name: lic.name,
            expiry: lic.expiry.map(|dt| dt.to_rfc3339()),
            status: lic.status,
            uses: lic.uses,
            max_machines: lic.max_machines,
            max_cores: lic.max_cores,
            max_uses: lic.max_uses,
            max_processes: lic.max_processes,
            max_users: lic.max_users,
            protected: lic.protected,
            suspended: lic.suspended,
            permissions: lic.permissions,
            policy: lic.policy,
            metadata: serde_json::to_value(lic.metadata).unwrap_or_default(),
            account_id: lic.account_id,
            product_id: lic.product_id,
            group_id: lic.group_id,
            owner_id: lic.owner_id,
        }
    }
}

fn make_license(id: String) -> keygen_rs::license::License {
    let mut lic = keygen_rs::license::License::from_key("");
    lic.id = id;
    lic
}

#[wasm_bindgen(js_name = "validate")]
pub async fn validate(fingerprints: JsValue, entitlements: JsValue) -> Result<JsValue, JsError> {
    let fps: Vec<String> =
        serde_wasm_bindgen::from_value(fingerprints).map_err(|e| JsError::new(&e.to_string()))?;
    let ents: Vec<String> =
        serde_wasm_bindgen::from_value(entitlements).map_err(|e| JsError::new(&e.to_string()))?;
    let license = keygen_rs::validate(&fps, &ents)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "verify")]
pub fn verify(scheme: String, signed_key: String) -> Result<Vec<u8>, JsError> {
    let scheme_code: keygen_rs::license::SchemeCode =
        serde_json::from_value(serde_json::Value::String(scheme))
            .map_err(|e| JsError::new(&format!("Invalid scheme: {e}")))?;
    keygen_rs::verify(scheme_code, &signed_key).map_err(to_js_error)
}

#[wasm_bindgen(js_name = "createLicense")]
pub async fn create_license(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        policy_id: String,
        name: Option<String>,
        key: Option<String>,
        expiry: Option<String>,
        max_machines: Option<i32>,
        max_processes: Option<i32>,
        max_users: Option<i32>,
        max_cores: Option<i32>,
        max_uses: Option<i32>,
        protected: Option<bool>,
        suspended: Option<bool>,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
        owner_id: Option<String>,
        group_id: Option<String>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let mut r = keygen_rs::license::LicenseCreateRequest::new(req.policy_id);
    if let Some(v) = req.name {
        r = r.with_name(v);
    }
    if let Some(v) = req.key {
        r = r.with_key(v);
    }
    if let Some(v) = req.expiry {
        let dt = chrono::DateTime::parse_from_rfc3339(&v)
            .map_err(|e| JsError::new(&format!("Invalid expiry: {e}")))?
            .with_timezone(&chrono::Utc);
        r = r.with_expiry(dt);
    }
    if let Some(v) = req.max_machines {
        r = r.with_max_machines(v);
    }
    if let Some(v) = req.max_processes {
        r = r.with_max_processes(v);
    }
    if let Some(v) = req.max_users {
        r = r.with_max_users(v);
    }
    if let Some(v) = req.max_cores {
        r = r.with_max_cores(v);
    }
    if let Some(v) = req.max_uses {
        r = r.with_max_uses(v);
    }
    if let Some(v) = req.protected {
        r = r.with_protected(v);
    }
    if let Some(v) = req.suspended {
        r = r.with_suspended(v);
    }
    if let Some(v) = req.permissions {
        r = r.with_permissions(v);
    }
    if let Some(meta) = req.metadata {
        if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(meta) {
            r = r.with_metadata(map);
        }
    }
    if let Some(v) = req.owner_id {
        r = r.with_owner_id(v);
    }
    if let Some(v) = req.group_id {
        r = r.with_group_id(v);
    }

    let license = keygen_rs::license::License::create(r)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listLicenses")]
pub async fn list_licenses(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<i32>,
        page_size: Option<i32>,
        page_number: Option<i32>,
        status: Option<String>,
        product: Option<String>,
        policy: Option<String>,
        owner: Option<String>,
        user: Option<String>,
        group: Option<String>,
        machine: Option<String>,
        assigned: Option<bool>,
        unassigned: Option<bool>,
        activated: Option<bool>,
        metadata: Option<serde_json::Value>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };
    let list_opts = opts.map(|o| keygen_rs::license::LicenseListOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        status: o.status,
        product: o.product,
        policy: o.policy,
        owner: o.owner,
        user: o.user,
        group: o.group,
        machine: o.machine,
        assigned: o.assigned,
        unassigned: o.unassigned,
        activated: o.activated,
        metadata: o
            .metadata
            .and_then(|meta| serde_json::from_value(meta).ok()),
        ..Default::default()
    });
    let licenses: Vec<License> = keygen_rs::license::License::list(list_opts.as_ref())
        .await
        .map(|list| list.into_iter().map(License::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&licenses).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getLicense")]
pub async fn get_license(id: String) -> Result<JsValue, JsError> {
    let license = keygen_rs::license::License::get(&id)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateLicense")]
pub async fn update_license(id: String, request: JsValue) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let obj: serde_json::Value =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;
    let obj = obj
        .as_object()
        .ok_or_else(|| JsError::new("request must be an object"))?;

    let mut req = keygen_rs::license::LicenseUpdateRequest::new();

    if let Some(serde_json::Value::String(name)) = obj.get("name") {
        req = req.with_name(name.clone());
    }
    if let Some(v) = obj.get("expiry") {
        if let Some(s) = v.as_str() {
            let dt = chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|e| JsError::new(&format!("Invalid expiry: {e}")))?
                .with_timezone(&chrono::Utc);
            req = req.with_expiry(dt);
        }
    }

    macro_rules! apply_clearable {
        ($obj:expr, $req:expr, $field:literal, $set:ident, $clear:ident) => {
            if let Some(v) = $obj.get($field) {
                if v.is_null() {
                    $req = $req.$clear();
                } else if let Some(n) = v.as_i64() {
                    $req = $req.$set(n as i32);
                }
            }
        };
    }

    apply_clearable!(
        obj,
        req,
        "maxMachines",
        with_max_machines,
        clear_max_machines
    );
    apply_clearable!(
        obj,
        req,
        "maxProcesses",
        with_max_processes,
        clear_max_processes
    );
    apply_clearable!(obj, req, "maxUsers", with_max_users, clear_max_users);
    apply_clearable!(obj, req, "maxCores", with_max_cores, clear_max_cores);
    apply_clearable!(obj, req, "maxUses", with_max_uses, clear_max_uses);

    if let Some(serde_json::Value::Bool(protected)) = obj.get("protected") {
        req = req.with_protected(*protected);
    }
    if let Some(serde_json::Value::Bool(suspended)) = obj.get("suspended") {
        req = req.with_suspended(*suspended);
    }
    if let Some(serde_json::Value::Array(perms)) = obj.get("permissions") {
        let permissions: Vec<String> = perms
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        req = req.with_permissions(permissions);
    }
    if let Some(meta) = obj.get("metadata") {
        if !meta.is_null() {
            if let Ok(map) =
                serde_json::from_value::<HashMap<String, serde_json::Value>>(meta.clone())
            {
                req = req.with_metadata(map);
            }
        }
    }

    let license = lic
        .update(req)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteLicense")]
pub async fn delete_license(id: String) -> Result<(), JsError> {
    let lic = make_license(id);
    lic.delete().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "suspendLicense")]
pub async fn suspend_license(id: String) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let license = lic
        .suspend()
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "reinstateLicense")]
pub async fn reinstate_license(id: String) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let license = lic
        .reinstate()
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "renewLicense")]
pub async fn renew_license(id: String) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let license = lic.renew().await.map(License::from).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "revokeLicense")]
pub async fn revoke_license(id: String) -> Result<(), JsError> {
    let lic = make_license(id);
    lic.revoke().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "incrementLicenseUsage")]
pub async fn increment_license_usage(id: String) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let license = lic
        .increment_usage()
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "decrementLicenseUsage")]
pub async fn decrement_license_usage(id: String) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let license = lic
        .decrement_usage()
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "resetLicenseUsage")]
pub async fn reset_license_usage(id: String) -> Result<JsValue, JsError> {
    let lic = make_license(id);
    let license = lic
        .reset_usage()
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "checkoutLicense")]
pub async fn checkout_license(id: String, opts: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        ttl: Option<i64>,
        include: Option<Vec<String>>,
    }
    let opts: Option<Opts> = if opts.is_undefined() || opts.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(opts).map_err(|e| JsError::new(&e.to_string()))?)
    };
    let lic = make_license(id);
    let checkout_opts = keygen_rs::license::LicenseCheckoutOpts {
        ttl: opts.as_ref().and_then(|o| o.ttl),
        include: opts.and_then(|o| o.include),
    };
    let lf = lic
        .checkout(&checkout_opts)
        .await
        .map(LicenseFile::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&lf).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "attachLicenseEntitlements")]
pub async fn attach_license_entitlements(
    id: String,
    entitlement_ids: JsValue,
) -> Result<(), JsError> {
    let ids: Vec<String> = serde_wasm_bindgen::from_value(entitlement_ids)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let lic = make_license(id);
    lic.attach_entitlements(&ids).await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "detachLicenseEntitlements")]
pub async fn detach_license_entitlements(
    id: String,
    entitlement_ids: JsValue,
) -> Result<(), JsError> {
    let ids: Vec<String> = serde_wasm_bindgen::from_value(entitlement_ids)
        .map_err(|e| JsError::new(&e.to_string()))?;
    let lic = make_license(id);
    lic.detach_entitlements(&ids).await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "checkInLicense")]
pub async fn check_in_license(id: String) -> Result<JsValue, JsError> {
    let license = make_license(id)
        .check_in()
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "generateLicenseToken")]
pub async fn generate_license_token(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        expiry: Option<String>,
        permissions: Option<Vec<String>>,
        metadata: Option<serde_json::Value>,
    }

    let req: Option<Req> = if request.is_undefined() || request.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let req = req
        .map(
            |request| -> Result<keygen_rs::token::CreateTokenRequest, JsError> {
                Ok(keygen_rs::token::CreateTokenRequest {
                    name: request.name,
                    expiry: request.expiry,
                    permissions: request.permissions,
                    metadata: crate::opt_metadata(request.metadata)?,
                })
            },
        )
        .transpose()?;

    let token = make_license(id)
        .generate_token(req)
        .await
        .map(Token::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&token).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "attachLicenseUsers")]
pub async fn attach_license_users(id: String, user_ids: JsValue) -> Result<(), JsError> {
    let ids: Vec<String> =
        serde_wasm_bindgen::from_value(user_ids).map_err(|e| JsError::new(&e.to_string()))?;
    make_license(id)
        .attach_users(&ids)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen(js_name = "detachLicenseUsers")]
pub async fn detach_license_users(id: String, user_ids: JsValue) -> Result<(), JsError> {
    let ids: Vec<String> =
        serde_wasm_bindgen::from_value(user_ids).map_err(|e| JsError::new(&e.to_string()))?;
    make_license(id)
        .detach_users(&ids)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen(js_name = "listLicenseUsers")]
pub async fn list_license_users(id: String) -> Result<JsValue, JsError> {
    let users: Vec<User> = make_license(id)
        .users(None)
        .await
        .map(|users| users.into_iter().map(User::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&users).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "changeLicensePolicy")]
pub async fn change_license_policy(id: String, policy_id: String) -> Result<JsValue, JsError> {
    let license = make_license(id)
        .change_policy(&policy_id)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "changeLicenseOwner")]
pub async fn change_license_owner(id: String, owner_id: String) -> Result<JsValue, JsError> {
    let license = make_license(id)
        .change_owner(&owner_id)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "changeLicenseGroup")]
pub async fn change_license_group(id: String, group_id: String) -> Result<JsValue, JsError> {
    let license = make_license(id)
        .change_group(&group_id)
        .await
        .map(License::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&license).map_err(|e| JsError::new(&e.to_string()))
}
