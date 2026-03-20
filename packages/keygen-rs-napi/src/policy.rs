use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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
    val.as_ref().and_then(|v| crate::enum_to_string(v))
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
            expiration_strategy: crate::enum_to_string(&p.expiration_strategy),
            expiration_basis: p.expiration_basis,
            renewal_basis: p.renewal_basis,
            authentication_strategy: crate::enum_to_string(&p.authentication_strategy),
            machine_leasing_strategy: crate::enum_to_string(&p.machine_leasing_strategy),
            process_leasing_strategy: crate::enum_to_string(&p.process_leasing_strategy),
            overage_strategy: crate::enum_to_string(&p.overage_strategy),
            transfer_strategy: crate::enum_to_string(&p.transfer_strategy),
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

#[napi(object)]
#[derive(Clone)]
pub struct CreatePolicyRequest {
    pub product_id: String,
    pub name: String,
    pub duration: Option<i64>,
    pub strict: Option<bool>,
    pub floating: Option<bool>,
    pub require_heartbeat: Option<bool>,
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
    pub encrypted: Option<bool>,
    pub protected: Option<bool>,
    pub require_check_in: Option<bool>,
    pub check_in_interval: Option<String>,
    pub check_in_interval_count: Option<i32>,
    pub use_pool: Option<bool>,
    pub max_licenses: Option<i32>,
    pub max_users: Option<i32>,
    pub scheme: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdatePolicyRequest {
    pub name: Option<String>,
    pub duration: Option<i64>,
    pub strict: Option<bool>,
    pub floating: Option<bool>,
    pub require_heartbeat: Option<bool>,
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
    pub protected: Option<bool>,
    pub require_check_in: Option<bool>,
    pub check_in_interval: Option<String>,
    pub check_in_interval_count: Option<i32>,
    pub max_users: Option<i32>,
    pub scheme: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListPoliciesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub product: Option<String>,
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

use crate::parse_enum;
use crate::to_metadata;

#[napi]
pub async fn create_policy(request: CreatePolicyRequest) -> Result<Policy> {
    let req = keygen_rs::policy::CreatePolicyRequest {
        product_id: request.product_id,
        name: request.name,
        duration: request.duration,
        strict: request.strict,
        floating: request.floating,
        require_heartbeat: request.require_heartbeat,
        heartbeat_duration: request.heartbeat_duration,
        heartbeat_cull_strategy: request.heartbeat_cull_strategy,
        heartbeat_resurrection_strategy: request.heartbeat_resurrection_strategy,
        heartbeat_basis: request.heartbeat_basis,
        machine_uniqueness_strategy: request
            .machine_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine uniqueness strategy"))
            .transpose()?,
        component_uniqueness_strategy: request
            .component_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component uniqueness strategy"))
            .transpose()?,
        machine_matching_strategy: request
            .machine_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine matching strategy"))
            .transpose()?,
        component_matching_strategy: request
            .component_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component matching strategy"))
            .transpose()?,
        expiration_strategy: request
            .expiration_strategy
            .as_deref()
            .map(|s| parse_enum(s, "expiration strategy"))
            .transpose()?,
        expiration_basis: request.expiration_basis,
        renewal_basis: request.renewal_basis,
        authentication_strategy: request
            .authentication_strategy
            .as_deref()
            .map(|s| parse_enum(s, "authentication strategy"))
            .transpose()?,
        machine_leasing_strategy: request
            .machine_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine leasing strategy"))
            .transpose()?,
        process_leasing_strategy: request
            .process_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "process leasing strategy"))
            .transpose()?,
        overage_strategy: request
            .overage_strategy
            .as_deref()
            .map(|s| parse_enum(s, "overage strategy"))
            .transpose()?,
        transfer_strategy: request
            .transfer_strategy
            .as_deref()
            .map(|s| parse_enum(s, "transfer strategy"))
            .transpose()?,
        max_machines: request.max_machines,
        max_processes: request.max_processes,
        max_cores: request.max_cores,
        max_uses: request.max_uses,
        encrypted: request.encrypted,
        protected: request.protected,
        require_check_in: request.require_check_in,
        check_in_interval: request.check_in_interval,
        check_in_interval_count: request.check_in_interval_count,
        use_pool: request.use_pool,
        max_licenses: request.max_licenses,
        max_users: request.max_users,
        scheme: request
            .scheme
            .as_deref()
            .map(|s| parse_enum(s, "scheme"))
            .transpose()?,
        metadata: request.metadata.map(to_metadata).transpose()?,
    };

    keygen_rs::policy::Policy::create(req)
        .await
        .map(Policy::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_policies(options: Option<ListPoliciesOptions>) -> Result<Vec<Policy>> {
    let opts = options.map(|o| keygen_rs::policy::ListPoliciesOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        product: o.product,
    });

    keygen_rs::policy::Policy::list(opts)
        .await
        .map(|ps| ps.into_iter().map(Policy::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_policy(id: String) -> Result<Policy> {
    keygen_rs::policy::Policy::get(&id)
        .await
        .map(Policy::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_policy(id: String, request: UpdatePolicyRequest) -> Result<Policy> {
    let policy = make_policy(id);

    let req = keygen_rs::policy::UpdatePolicyRequest {
        name: request.name,
        duration: request.duration,
        strict: request.strict,
        floating: request.floating,
        require_heartbeat: request.require_heartbeat,
        heartbeat_duration: request.heartbeat_duration,
        heartbeat_cull_strategy: request.heartbeat_cull_strategy,
        heartbeat_resurrection_strategy: request.heartbeat_resurrection_strategy,
        heartbeat_basis: request.heartbeat_basis,
        machine_uniqueness_strategy: request
            .machine_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine uniqueness strategy"))
            .transpose()?,
        component_uniqueness_strategy: request
            .component_uniqueness_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component uniqueness strategy"))
            .transpose()?,
        machine_matching_strategy: request
            .machine_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine matching strategy"))
            .transpose()?,
        component_matching_strategy: request
            .component_matching_strategy
            .as_deref()
            .map(|s| parse_enum(s, "component matching strategy"))
            .transpose()?,
        expiration_strategy: request
            .expiration_strategy
            .as_deref()
            .map(|s| parse_enum(s, "expiration strategy"))
            .transpose()?,
        expiration_basis: request.expiration_basis,
        renewal_basis: request.renewal_basis,
        authentication_strategy: request
            .authentication_strategy
            .as_deref()
            .map(|s| parse_enum(s, "authentication strategy"))
            .transpose()?,
        machine_leasing_strategy: request
            .machine_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "machine leasing strategy"))
            .transpose()?,
        process_leasing_strategy: request
            .process_leasing_strategy
            .as_deref()
            .map(|s| parse_enum(s, "process leasing strategy"))
            .transpose()?,
        overage_strategy: request
            .overage_strategy
            .as_deref()
            .map(|s| parse_enum(s, "overage strategy"))
            .transpose()?,
        transfer_strategy: request
            .transfer_strategy
            .as_deref()
            .map(|s| parse_enum(s, "transfer strategy"))
            .transpose()?,
        max_machines: request.max_machines,
        max_processes: request.max_processes,
        max_cores: request.max_cores,
        max_uses: request.max_uses,
        protected: request.protected,
        require_check_in: request.require_check_in,
        check_in_interval: request.check_in_interval,
        check_in_interval_count: request.check_in_interval_count,
        max_users: request.max_users,
        scheme: request
            .scheme
            .as_deref()
            .map(|s| parse_enum(s, "scheme"))
            .transpose()?,
        metadata: request.metadata.map(to_metadata).transpose()?,
    };

    policy
        .update(req)
        .await
        .map(Policy::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_policy(id: String) -> Result<()> {
    let policy = make_policy(id);
    policy.delete().await.map_err(to_napi_error)
}
