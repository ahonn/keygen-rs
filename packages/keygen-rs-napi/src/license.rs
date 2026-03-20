use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::license_file::LicenseFile;
use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct LicenseCheckoutOpts {
    pub ttl: Option<i64>,
    pub include: Option<Vec<String>>,
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseCreateRequest {
    pub policy_id: String,
    pub name: Option<String>,
    pub key: Option<String>,
    pub expiry: Option<String>,
    pub max_machines: Option<i32>,
    pub max_processes: Option<i32>,
    pub max_users: Option<i32>,
    pub max_cores: Option<i32>,
    pub max_uses: Option<i32>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
}

// LicenseUpdateRequest is accepted as serde_json::Value in update_license()
// to distinguish null (clear) from undefined (keep) for clearable fields.
// The TypeScript type is specified via #[napi(ts_args_type)] on the function.

#[napi(object)]
#[derive(Clone)]
pub struct ListLicensesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub status: Option<String>,
    pub product: Option<String>,
    pub policy: Option<String>,
    pub owner: Option<String>,
    pub user: Option<String>,
}

fn make_license(id: String) -> keygen_rs::license::License {
    let mut lic = keygen_rs::license::License::from_key("");
    lic.id = id;
    lic
}

#[napi]
pub async fn validate(
    fingerprints: Vec<String>,
    entitlements: Option<Vec<String>>,
) -> Result<License> {
    let entitlements = entitlements.unwrap_or_default();
    keygen_rs::validate(&fingerprints, &entitlements)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub fn verify(scheme: String, signed_key: String) -> Result<Buffer> {
    let scheme_code: keygen_rs::license::SchemeCode =
        serde_json::from_value(serde_json::Value::String(scheme))
            .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid scheme: {e}")))?;

    keygen_rs::verify(scheme_code, &signed_key)
        .map(|bytes| bytes.into())
        .map_err(to_napi_error)
}

#[napi]
pub async fn create_license(request: LicenseCreateRequest) -> Result<License> {
    let mut req = keygen_rs::license::LicenseCreateRequest::new(request.policy_id);

    if let Some(name) = request.name {
        req = req.with_name(name);
    }
    if let Some(key) = request.key {
        req = req.with_key(key);
    }
    if let Some(expiry) = request.expiry {
        let dt = chrono::DateTime::parse_from_rfc3339(&expiry)
            .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid expiry: {e}")))?
            .with_timezone(&chrono::Utc);
        req = req.with_expiry(dt);
    }
    if let Some(max_machines) = request.max_machines {
        req = req.with_max_machines(max_machines);
    }
    if let Some(max_processes) = request.max_processes {
        req = req.with_max_processes(max_processes);
    }
    if let Some(max_users) = request.max_users {
        req = req.with_max_users(max_users);
    }
    if let Some(max_cores) = request.max_cores {
        req = req.with_max_cores(max_cores);
    }
    if let Some(max_uses) = request.max_uses {
        req = req.with_max_uses(max_uses);
    }
    if let Some(protected) = request.protected {
        req = req.with_protected(protected);
    }
    if let Some(suspended) = request.suspended {
        req = req.with_suspended(suspended);
    }
    if let Some(permissions) = request.permissions {
        req = req.with_permissions(permissions);
    }
    if let Some(meta) = request.metadata {
        if let Ok(map) = serde_json::from_value::<HashMap<String, serde_json::Value>>(meta) {
            req = req.with_metadata(map);
        }
    }
    if let Some(owner_id) = request.owner_id {
        req = req.with_owner_id(owner_id);
    }
    if let Some(group_id) = request.group_id {
        req = req.with_group_id(group_id);
    }

    keygen_rs::license::License::create(req)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_licenses(options: Option<ListLicensesOptions>) -> Result<Vec<License>> {
    let opts = options.map(|o| keygen_rs::license::LicenseListOptions {
        limit: o.limit.map(|v| v as i32),
        page_size: o.page_size.map(|v| v as i32),
        page_number: o.page_number.map(|v| v as i32),
        status: o.status,
        product: o.product,
        policy: o.policy,
        owner: o.owner,
        user: o.user,
    });

    keygen_rs::license::License::list(opts.as_ref())
        .await
        .map(|licenses| licenses.into_iter().map(License::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_license(id: String) -> Result<License> {
    keygen_rs::license::License::get(&id)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

/// Update a license. Clearable integer fields accept `number | null`:
/// - `undefined` / absent → keep unchanged
/// - `null` → clear (set to null)
/// - `number` → set to value
#[napi(
    ts_args_type = "id: string, request: { name?: string; expiry?: string; maxMachines?: number | null; maxProcesses?: number | null; maxUsers?: number | null; maxCores?: number | null; maxUses?: number | null; protected?: boolean; suspended?: boolean; permissions?: string[]; metadata?: any }"
)]
pub async fn update_license(id: String, request: serde_json::Value) -> Result<License> {
    let lic = make_license(id);

    let obj = request
        .as_object()
        .ok_or_else(|| napi::Error::new(Status::InvalidArg, "request must be an object"))?;

    let mut req = keygen_rs::license::LicenseUpdateRequest::new();

    if let Some(serde_json::Value::String(name)) = obj.get("name") {
        req = req.with_name(name.clone());
    }
    if let Some(v) = obj.get("expiry") {
        if let Some(s) = v.as_str() {
            let dt = chrono::DateTime::parse_from_rfc3339(s)
                .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid expiry: {e}")))?
                .with_timezone(&chrono::Utc);
            req = req.with_expiry(dt);
        }
    }

    // Clearable integer fields: null → clear, number → set, absent → keep
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

    lic.update(req)
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_license(id: String) -> Result<()> {
    let lic = make_license(id);
    lic.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn suspend_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.suspend()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn reinstate_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.reinstate()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn renew_license(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.renew().await.map(License::from).map_err(to_napi_error)
}

#[napi]
pub async fn revoke_license(id: String) -> Result<()> {
    let lic = make_license(id);
    lic.revoke().await.map_err(to_napi_error)
}

#[napi]
pub async fn increment_license_usage(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.increment_usage()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn decrement_license_usage(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.decrement_usage()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn reset_license_usage(id: String) -> Result<License> {
    let lic = make_license(id);
    lic.reset_usage()
        .await
        .map(License::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn checkout_license(
    id: String,
    opts: Option<LicenseCheckoutOpts>,
) -> Result<LicenseFile> {
    let lic = make_license(id);
    let checkout_opts = keygen_rs::license::LicenseCheckoutOpts {
        ttl: opts.as_ref().and_then(|o| o.ttl),
        include: opts.and_then(|o| o.include),
    };

    lic.checkout(&checkout_opts)
        .await
        .map(LicenseFile::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn attach_license_entitlements(id: String, entitlement_ids: Vec<String>) -> Result<()> {
    let lic = make_license(id);
    lic.attach_entitlements(&entitlement_ids)
        .await
        .map_err(to_napi_error)
}

#[napi]
pub async fn detach_license_entitlements(id: String, entitlement_ids: Vec<String>) -> Result<()> {
    let lic = make_license(id);
    lic.detach_entitlements(&entitlement_ids)
        .await
        .map_err(to_napi_error)
}
