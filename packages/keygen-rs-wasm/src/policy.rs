use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::entitlement::Entitlement;
use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub duration: Option<i64>,
    pub strict: bool,
    pub floating: bool,
    pub require_heartbeat: bool,
    pub heartbeat_duration: Option<i64>,
    pub heartbeat_cull_strategy: Option<String>,
    pub heartbeat_resurrection_strategy: Option<String>,
    pub heartbeat_basis: Option<String>,
    pub machine_uniqueness_strategy: Option<String>,
    pub component_uniqueness_strategy: Option<String>,
    pub machine_matching_strategy: Option<String>,
    pub component_matching_strategy: Option<String>,
    pub expiration_strategy: Option<String>,
    pub expiration_basis: Option<String>,
    pub renewal_basis: Option<String>,
    pub authentication_strategy: Option<String>,
    pub machine_leasing_strategy: Option<String>,
    pub process_leasing_strategy: Option<String>,
    pub overage_strategy: Option<String>,
    pub transfer_strategy: Option<String>,
    pub max_machines: Option<i32>,
    pub max_processes: Option<i32>,
    pub max_cores: Option<i32>,
    pub max_uses: Option<i32>,
    pub encrypted: bool,
    pub protected: bool,
    pub require_check_in: bool,
    pub check_in_interval: Option<String>,
    pub check_in_interval_count: Option<i32>,
    pub use_pool: bool,
    pub max_licenses: Option<i32>,
    pub max_users: Option<i32>,
    pub scheme: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub product_id: Option<String>,
}

fn opt_enum_to_string<T: serde::Serialize>(val: &Option<T>) -> Option<String> {
    val.as_ref()
        .and_then(|v| serde_json::to_value(v).ok())
        .and_then(|v| v.as_str().map(String::from))
}

fn enum_to_string<T: serde::Serialize>(val: &T) -> Option<String> {
    serde_json::to_value(val)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str, label: &str) -> Result<T, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid {label}: {e}")))
}

impl From<keygen_rs::policy::Policy> for Policy {
    fn from(p: keygen_rs::policy::Policy) -> Self {
        Policy {
            id: p.id,
            name: p.name,
            duration: p.duration,
            strict: p.strict,
            floating: p.floating,
            require_heartbeat: p.require_heartbeat,
            heartbeat_duration: p.heartbeat_duration,
            heartbeat_cull_strategy: p.heartbeat_cull_strategy,
            heartbeat_resurrection_strategy: p.heartbeat_resurrection_strategy,
            heartbeat_basis: p.heartbeat_basis,
            machine_uniqueness_strategy: opt_enum_to_string(&p.machine_uniqueness_strategy),
            component_uniqueness_strategy: opt_enum_to_string(&p.component_uniqueness_strategy),
            machine_matching_strategy: opt_enum_to_string(&p.machine_matching_strategy),
            component_matching_strategy: opt_enum_to_string(&p.component_matching_strategy),
            expiration_strategy: enum_to_string(&p.expiration_strategy),
            expiration_basis: p.expiration_basis,
            renewal_basis: p.renewal_basis,
            authentication_strategy: enum_to_string(&p.authentication_strategy),
            machine_leasing_strategy: enum_to_string(&p.machine_leasing_strategy),
            process_leasing_strategy: enum_to_string(&p.process_leasing_strategy),
            overage_strategy: enum_to_string(&p.overage_strategy),
            transfer_strategy: enum_to_string(&p.transfer_strategy),
            max_machines: p.max_machines,
            max_processes: p.max_processes,
            max_cores: p.max_cores,
            max_uses: p.max_uses,
            encrypted: p.encrypted,
            protected: p.protected,
            require_check_in: p.require_check_in,
            check_in_interval: p.check_in_interval,
            check_in_interval_count: p.check_in_interval_count,
            use_pool: p.use_pool,
            max_licenses: p.max_licenses,
            max_users: p.max_users,
            scheme: opt_enum_to_string(&p.scheme),
            metadata: p
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: p.created,
            updated: p.updated,
            account_id: p.account_id,
            product_id: p.product_id,
        }
    }
}

fn make_policy(id: String) -> keygen_rs::policy::Policy {
    use keygen_rs::policy::*;

    Policy {
        id,
        name: String::new(),
        duration: None,
        strict: false,
        floating: false,
        require_heartbeat: false,
        heartbeat_duration: None,
        heartbeat_cull_strategy: None,
        heartbeat_resurrection_strategy: None,
        heartbeat_basis: None,
        machine_uniqueness_strategy: None,
        component_uniqueness_strategy: None,
        machine_matching_strategy: None,
        component_matching_strategy: None,
        expiration_strategy: ExpirationStrategy::RestrictAccess,
        expiration_basis: None,
        renewal_basis: None,
        authentication_strategy: AuthenticationStrategy::Token,
        machine_leasing_strategy: LeasingStrategy::PerMachine,
        process_leasing_strategy: LeasingStrategy::PerMachine,
        overage_strategy: OverageStrategy::NoOverage,
        transfer_strategy: TransferStrategy::KeepPolicy,
        max_machines: None,
        max_processes: None,
        max_cores: None,
        max_uses: None,
        encrypted: false,
        protected: false,
        require_check_in: false,
        check_in_interval: None,
        check_in_interval_count: None,
        use_pool: false,
        max_licenses: None,
        max_users: None,
        scheme: None,
        metadata: None,
        created: String::new(),
        updated: String::new(),
        account_id: None,
        product_id: None,
    }
}

#[wasm_bindgen(js_name = "createPolicy")]
pub async fn create_policy(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        product_id: String,
        name: String,
        duration: Option<i64>,
        strict: Option<bool>,
        floating: Option<bool>,
        require_heartbeat: Option<bool>,
        heartbeat_duration: Option<i64>,
        heartbeat_cull_strategy: Option<String>,
        heartbeat_resurrection_strategy: Option<String>,
        heartbeat_basis: Option<String>,
        machine_uniqueness_strategy: Option<String>,
        component_uniqueness_strategy: Option<String>,
        machine_matching_strategy: Option<String>,
        component_matching_strategy: Option<String>,
        expiration_strategy: Option<String>,
        expiration_basis: Option<String>,
        renewal_basis: Option<String>,
        authentication_strategy: Option<String>,
        machine_leasing_strategy: Option<String>,
        process_leasing_strategy: Option<String>,
        overage_strategy: Option<String>,
        transfer_strategy: Option<String>,
        max_machines: Option<i32>,
        max_processes: Option<i32>,
        max_cores: Option<i32>,
        max_uses: Option<i32>,
        encrypted: Option<bool>,
        protected: Option<bool>,
        require_check_in: Option<bool>,
        check_in_interval: Option<String>,
        check_in_interval_count: Option<i32>,
        use_pool: Option<bool>,
        max_licenses: Option<i32>,
        max_users: Option<i32>,
        scheme: Option<String>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::policy::CreatePolicyRequest {
        product_id: req.product_id,
        name: req.name,
        duration: req.duration,
        strict: req.strict,
        floating: req.floating,
        require_heartbeat: req.require_heartbeat,
        heartbeat_duration: req.heartbeat_duration,
        heartbeat_cull_strategy: req.heartbeat_cull_strategy,
        heartbeat_resurrection_strategy: req.heartbeat_resurrection_strategy,
        heartbeat_basis: req.heartbeat_basis,
        machine_uniqueness_strategy: req
            .machine_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine uniqueness strategy"))
            .transpose()?,
        component_uniqueness_strategy: req
            .component_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component uniqueness strategy"))
            .transpose()?,
        machine_matching_strategy: req
            .machine_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine matching strategy"))
            .transpose()?,
        component_matching_strategy: req
            .component_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component matching strategy"))
            .transpose()?,
        expiration_strategy: req
            .expiration_strategy
            .as_deref()
            .map(|s| parse_enum(s, "expiration strategy"))
            .transpose()?,
        expiration_basis: req.expiration_basis,
        renewal_basis: req.renewal_basis,
        authentication_strategy: req
            .authentication_strategy
            .as_deref()
            .map(|s| parse_enum(s, "authentication strategy"))
            .transpose()?,
        machine_leasing_strategy: req
            .machine_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine leasing strategy"))
            .transpose()?,
        process_leasing_strategy: req
            .process_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "process leasing strategy"))
            .transpose()?,
        overage_strategy: req
            .overage_strategy
            .as_deref()
            .map(|s| parse_enum(s, "overage strategy"))
            .transpose()?,
        transfer_strategy: req
            .transfer_strategy
            .as_deref()
            .map(|s| parse_enum(s, "transfer strategy"))
            .transpose()?,
        max_machines: req.max_machines,
        max_processes: req.max_processes,
        max_cores: req.max_cores,
        max_uses: req.max_uses,
        encrypted: req.encrypted,
        protected: req.protected,
        require_check_in: req.require_check_in,
        check_in_interval: req.check_in_interval,
        check_in_interval_count: req.check_in_interval_count,
        use_pool: req.use_pool,
        max_licenses: req.max_licenses,
        max_users: req.max_users,
        scheme: req
            .scheme
            .as_deref()
            .map(|s| parse_enum(s, "scheme"))
            .transpose()?,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let policy = keygen_rs::policy::Policy::create(r)
        .await
        .map(Policy::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&policy).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listPolicies")]
pub async fn list_policies(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
        product: Option<String>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let list_opts = opts.map(|o| keygen_rs::policy::ListPoliciesOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        product: o.product,
    });

    let policies: Vec<Policy> = keygen_rs::policy::Policy::list(list_opts)
        .await
        .map(|ps| ps.into_iter().map(Policy::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&policies).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getPolicy")]
pub async fn get_policy(id: String) -> Result<JsValue, JsError> {
    let policy = keygen_rs::policy::Policy::get(&id)
        .await
        .map(Policy::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&policy).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updatePolicy")]
pub async fn update_policy(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        duration: Option<i64>,
        strict: Option<bool>,
        floating: Option<bool>,
        require_heartbeat: Option<bool>,
        heartbeat_duration: Option<i64>,
        heartbeat_cull_strategy: Option<String>,
        heartbeat_resurrection_strategy: Option<String>,
        heartbeat_basis: Option<String>,
        machine_uniqueness_strategy: Option<String>,
        component_uniqueness_strategy: Option<String>,
        machine_matching_strategy: Option<String>,
        component_matching_strategy: Option<String>,
        expiration_strategy: Option<String>,
        expiration_basis: Option<String>,
        renewal_basis: Option<String>,
        authentication_strategy: Option<String>,
        machine_leasing_strategy: Option<String>,
        process_leasing_strategy: Option<String>,
        overage_strategy: Option<String>,
        transfer_strategy: Option<String>,
        max_machines: Option<i32>,
        max_processes: Option<i32>,
        max_cores: Option<i32>,
        max_uses: Option<i32>,
        protected: Option<bool>,
        require_check_in: Option<bool>,
        check_in_interval: Option<String>,
        check_in_interval_count: Option<i32>,
        max_users: Option<i32>,
        scheme: Option<String>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let p = make_policy(id);

    let r = keygen_rs::policy::UpdatePolicyRequest {
        name: req.name,
        duration: req.duration,
        strict: req.strict,
        floating: req.floating,
        require_heartbeat: req.require_heartbeat,
        heartbeat_duration: req.heartbeat_duration,
        heartbeat_cull_strategy: req.heartbeat_cull_strategy,
        heartbeat_resurrection_strategy: req.heartbeat_resurrection_strategy,
        heartbeat_basis: req.heartbeat_basis,
        machine_uniqueness_strategy: req
            .machine_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine uniqueness strategy"))
            .transpose()?,
        component_uniqueness_strategy: req
            .component_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component uniqueness strategy"))
            .transpose()?,
        machine_matching_strategy: req
            .machine_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine matching strategy"))
            .transpose()?,
        component_matching_strategy: req
            .component_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component matching strategy"))
            .transpose()?,
        expiration_strategy: req
            .expiration_strategy
            .as_deref()
            .map(|s| parse_enum(s, "expiration strategy"))
            .transpose()?,
        expiration_basis: req.expiration_basis,
        renewal_basis: req.renewal_basis,
        authentication_strategy: req
            .authentication_strategy
            .as_deref()
            .map(|s| parse_enum(s, "authentication strategy"))
            .transpose()?,
        machine_leasing_strategy: req
            .machine_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine leasing strategy"))
            .transpose()?,
        process_leasing_strategy: req
            .process_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "process leasing strategy"))
            .transpose()?,
        overage_strategy: req
            .overage_strategy
            .as_deref()
            .map(|s| parse_enum(s, "overage strategy"))
            .transpose()?,
        transfer_strategy: req
            .transfer_strategy
            .as_deref()
            .map(|s| parse_enum(s, "transfer strategy"))
            .transpose()?,
        max_machines: req.max_machines,
        max_processes: req.max_processes,
        max_cores: req.max_cores,
        max_uses: req.max_uses,
        protected: req.protected,
        require_check_in: req.require_check_in,
        check_in_interval: req.check_in_interval,
        check_in_interval_count: req.check_in_interval_count,
        max_users: req.max_users,
        scheme: req
            .scheme
            .as_deref()
            .map(|s| parse_enum(s, "scheme"))
            .transpose()?,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let policy = p.update(r).await.map(Policy::from).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&policy).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deletePolicy")]
pub async fn delete_policy(id: String) -> Result<(), JsError> {
    let policy = make_policy(id);
    policy.delete().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "attachPolicyEntitlements")]
pub async fn attach_policy_entitlements(
    id: String,
    entitlement_ids: JsValue,
) -> Result<(), JsError> {
    let ids: Vec<String> = serde_wasm_bindgen::from_value(entitlement_ids)
        .map_err(|e| JsError::new(&e.to_string()))?;
    make_policy(id)
        .attach_entitlements(&ids)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen(js_name = "detachPolicyEntitlements")]
pub async fn detach_policy_entitlements(
    id: String,
    entitlement_ids: JsValue,
) -> Result<(), JsError> {
    let ids: Vec<String> = serde_wasm_bindgen::from_value(entitlement_ids)
        .map_err(|e| JsError::new(&e.to_string()))?;
    make_policy(id)
        .detach_entitlements(&ids)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen(js_name = "listPolicyEntitlements")]
pub async fn list_policy_entitlements(id: String) -> Result<JsValue, JsError> {
    let items: Vec<Entitlement> = make_policy(id)
        .entitlements(None)
        .await
        .map(|items| items.into_iter().map(Entitlement::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&items).map_err(|e| JsError::new(&e.to_string()))
}
