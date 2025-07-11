use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpirationStrategy {
    RestrictAccess,
    RevokeAccess,
    MaintainAccess,
    AllowAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuthenticationStrategy {
    Token,
    License,
    Mixed,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverageStrategy {
    NoOverage,
    AlwaysAllowOverage,
    Allow125xOverage,
    Allow15xOverage,
    Allow2xOverage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransferStrategy {
    KeepPolicy,
    ResetPolicy,
    KeepExpiry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LeasingStrategy {
    PerMachine,
    PerLicense,
    PerUser,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UniquenessStrategy {
    UniquePerAccount,
    UniquePerProduct,
    UniquePerPolicy,
    UniquePerLicense,
    UniquePerMachine,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatchingStrategy {
    MatchAny,
    MatchTwo,
    MatchMost,
    MatchAll,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Scheme {
    #[serde(rename = "ED25519_SIGN")]
    Ed25519Sign,
    #[serde(rename = "RSA_2048_PKCS1_PSS_SIGN_V2")]
    Rsa2048Pkcs1PssSignV2,
    #[serde(rename = "RSA_2048_PKCS1_SIGN_V2")]
    Rsa2048Pkcs1SignV2,
    #[serde(rename = "RSA_2048_PKCS1_ENCRYPT")]
    Rsa2048Pkcs1Encrypt,
    #[serde(rename = "RSA_2048_JWT_RS256")]
    Rsa2048JwtRs256,
    #[serde(rename = "LEGACY_ENCRYPT")]
    LegacyEncrypt,
    #[serde(rename = "RSA_2048_PKCS1_PSS_SIGN")]
    Rsa2048Pkcs1PssSign, // Deprecated
    #[serde(rename = "RSA_2048_PKCS1_SIGN")]
    Rsa2048Pkcs1Sign, // Deprecated
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAttributes {
    pub name: String,
    pub duration: Option<i64>, // Duration in seconds, null for perpetual
    pub strict: bool,
    pub floating: bool,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: bool,
    #[serde(rename = "heartbeatDuration")]
    pub heartbeat_duration: Option<i64>,
    #[serde(rename = "heartbeatCullStrategy")]
    pub heartbeat_cull_strategy: Option<String>,
    #[serde(rename = "heartbeatResurrectionStrategy")]
    pub heartbeat_resurrection_strategy: Option<String>,
    #[serde(rename = "heartbeatBasis")]
    pub heartbeat_basis: Option<String>,
    #[serde(rename = "machineUniquenessStrategy")]
    pub machine_uniqueness_strategy: Option<UniquenessStrategy>,
    #[serde(rename = "componentUniquenessStrategy")]
    pub component_uniqueness_strategy: Option<UniquenessStrategy>,
    #[serde(rename = "machineMatchingStrategy")]
    pub machine_matching_strategy: Option<MatchingStrategy>,
    #[serde(rename = "componentMatchingStrategy")]
    pub component_matching_strategy: Option<MatchingStrategy>,
    #[serde(rename = "expirationStrategy")]
    pub expiration_strategy: ExpirationStrategy,
    #[serde(rename = "expirationBasis")]
    pub expiration_basis: Option<String>,
    #[serde(rename = "renewalBasis")]
    pub renewal_basis: Option<String>,
    #[serde(rename = "authenticationStrategy")]
    pub authentication_strategy: AuthenticationStrategy,
    #[serde(rename = "machineLeasingStrategy")]
    pub machine_leasing_strategy: LeasingStrategy,
    #[serde(rename = "processLeasingStrategy")]
    pub process_leasing_strategy: LeasingStrategy,
    #[serde(rename = "overageStrategy")]
    pub overage_strategy: OverageStrategy,
    #[serde(rename = "transferStrategy")]
    pub transfer_strategy: TransferStrategy,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    #[serde(rename = "maxProcesses")]
    pub max_processes: Option<i32>,
    #[serde(rename = "maxCores")]
    pub max_cores: Option<i32>,
    #[serde(rename = "maxUses")]
    pub max_uses: Option<i32>,
    pub encrypted: bool,
    pub protected: bool,
    #[serde(rename = "requireCheckIn")]
    pub require_check_in: bool,
    #[serde(rename = "checkInInterval")]
    pub check_in_interval: Option<String>,
    #[serde(rename = "checkInIntervalCount")]
    pub check_in_interval_count: Option<i32>,
    #[serde(rename = "usePool")]
    pub use_pool: bool,
    #[serde(rename = "maxLicenses")]
    pub max_licenses: Option<i32>,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    pub scheme: Option<Scheme>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PolicyResponse {
    pub data: KeygenResponseData<PolicyAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PoliciesResponse {
    pub data: Vec<KeygenResponseData<PolicyAttributes>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub duration: Option<i64>,
    pub strict: Option<bool>,
    pub floating: Option<bool>,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: Option<bool>,
    #[serde(rename = "heartbeatDuration")]
    pub heartbeat_duration: Option<i64>,
    #[serde(rename = "heartbeatCullStrategy")]
    pub heartbeat_cull_strategy: Option<String>,
    #[serde(rename = "heartbeatResurrectionStrategy")]
    pub heartbeat_resurrection_strategy: Option<String>,
    #[serde(rename = "heartbeatBasis")]
    pub heartbeat_basis: Option<String>,
    #[serde(rename = "machineUniquenessStrategy")]
    pub machine_uniqueness_strategy: Option<UniquenessStrategy>,
    #[serde(rename = "componentUniquenessStrategy")]
    pub component_uniqueness_strategy: Option<UniquenessStrategy>,
    #[serde(rename = "machineMatchingStrategy")]
    pub machine_matching_strategy: Option<MatchingStrategy>,
    #[serde(rename = "componentMatchingStrategy")]
    pub component_matching_strategy: Option<MatchingStrategy>,
    #[serde(rename = "expirationStrategy")]
    pub expiration_strategy: Option<ExpirationStrategy>,
    #[serde(rename = "expirationBasis")]
    pub expiration_basis: Option<String>,
    #[serde(rename = "renewalBasis")]
    pub renewal_basis: Option<String>,
    #[serde(rename = "authenticationStrategy")]
    pub authentication_strategy: Option<AuthenticationStrategy>,
    #[serde(rename = "machineLeasingStrategy")]
    pub machine_leasing_strategy: Option<LeasingStrategy>,
    #[serde(rename = "processLeasingStrategy")]
    pub process_leasing_strategy: Option<LeasingStrategy>,
    #[serde(rename = "overageStrategy")]
    pub overage_strategy: Option<OverageStrategy>,
    #[serde(rename = "transferStrategy")]
    pub transfer_strategy: Option<TransferStrategy>,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    #[serde(rename = "maxProcesses")]
    pub max_processes: Option<i32>,
    #[serde(rename = "maxCores")]
    pub max_cores: Option<i32>,
    #[serde(rename = "maxUses")]
    pub max_uses: Option<i32>,
    pub encrypted: Option<bool>,
    pub protected: Option<bool>,
    #[serde(rename = "requireCheckIn")]
    pub require_check_in: Option<bool>,
    #[serde(rename = "checkInInterval")]
    pub check_in_interval: Option<String>,
    #[serde(rename = "checkInIntervalCount")]
    pub check_in_interval_count: Option<i32>,
    #[serde(rename = "usePool")]
    pub use_pool: Option<bool>,
    #[serde(rename = "maxLicenses")]
    pub max_licenses: Option<i32>,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    pub scheme: Option<Scheme>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    // Relationship to product
    pub product_id: String,
}

impl Default for CreatePolicyRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            duration: None,
            strict: None,
            floating: None,
            require_heartbeat: None,
            heartbeat_duration: None,
            heartbeat_cull_strategy: None,
            heartbeat_resurrection_strategy: None,
            heartbeat_basis: None,
            machine_uniqueness_strategy: None,
            component_uniqueness_strategy: None,
            machine_matching_strategy: None,
            component_matching_strategy: None,
            expiration_strategy: None,
            expiration_basis: None,
            renewal_basis: None,
            authentication_strategy: None,
            machine_leasing_strategy: None,
            process_leasing_strategy: None,
            overage_strategy: None,
            transfer_strategy: None,
            max_machines: None,
            max_processes: None,
            max_cores: None,
            max_uses: None,
            encrypted: None,
            protected: None,
            require_check_in: None,
            check_in_interval: None,
            check_in_interval_count: None,
            use_pool: None,
            max_licenses: None,
            max_users: None,
            scheme: None,
            metadata: None,
            product_id: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyRequest {
    pub name: Option<String>,
    pub duration: Option<i64>,
    pub strict: Option<bool>,
    pub floating: Option<bool>,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: Option<bool>,
    #[serde(rename = "heartbeatDuration")]
    pub heartbeat_duration: Option<i64>,
    #[serde(rename = "heartbeatCullStrategy")]
    pub heartbeat_cull_strategy: Option<String>,
    #[serde(rename = "heartbeatResurrectionStrategy")]
    pub heartbeat_resurrection_strategy: Option<String>,
    #[serde(rename = "heartbeatBasis")]
    pub heartbeat_basis: Option<String>,
    #[serde(rename = "machineUniquenessStrategy")]
    pub machine_uniqueness_strategy: Option<UniquenessStrategy>,
    #[serde(rename = "componentUniquenessStrategy")]
    pub component_uniqueness_strategy: Option<UniquenessStrategy>,
    #[serde(rename = "machineMatchingStrategy")]
    pub machine_matching_strategy: Option<MatchingStrategy>,
    #[serde(rename = "componentMatchingStrategy")]
    pub component_matching_strategy: Option<MatchingStrategy>,
    #[serde(rename = "expirationStrategy")]
    pub expiration_strategy: Option<ExpirationStrategy>,
    #[serde(rename = "expirationBasis")]
    pub expiration_basis: Option<String>,
    #[serde(rename = "renewalBasis")]
    pub renewal_basis: Option<String>,
    #[serde(rename = "authenticationStrategy")]
    pub authentication_strategy: Option<AuthenticationStrategy>,
    #[serde(rename = "machineLeasingStrategy")]
    pub machine_leasing_strategy: Option<LeasingStrategy>,
    #[serde(rename = "processLeasingStrategy")]
    pub process_leasing_strategy: Option<LeasingStrategy>,
    #[serde(rename = "overageStrategy")]
    pub overage_strategy: Option<OverageStrategy>,
    #[serde(rename = "transferStrategy")]
    pub transfer_strategy: Option<TransferStrategy>,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    #[serde(rename = "maxProcesses")]
    pub max_processes: Option<i32>,
    #[serde(rename = "maxCores")]
    pub max_cores: Option<i32>,
    #[serde(rename = "maxUses")]
    pub max_uses: Option<i32>,
    pub protected: Option<bool>,
    #[serde(rename = "requireCheckIn")]
    pub require_check_in: Option<bool>,
    #[serde(rename = "checkInInterval")]
    pub check_in_interval: Option<String>,
    #[serde(rename = "checkInIntervalCount")]
    pub check_in_interval_count: Option<i32>,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    pub scheme: Option<Scheme>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Default for UpdatePolicyRequest {
    fn default() -> Self {
        Self {
            name: None,
            duration: None,
            strict: None,
            floating: None,
            require_heartbeat: None,
            heartbeat_duration: None,
            heartbeat_cull_strategy: None,
            heartbeat_resurrection_strategy: None,
            heartbeat_basis: None,
            machine_uniqueness_strategy: None,
            component_uniqueness_strategy: None,
            machine_matching_strategy: None,
            component_matching_strategy: None,
            expiration_strategy: None,
            expiration_basis: None,
            renewal_basis: None,
            authentication_strategy: None,
            machine_leasing_strategy: None,
            process_leasing_strategy: None,
            overage_strategy: None,
            transfer_strategy: None,
            max_machines: None,
            max_processes: None,
            max_cores: None,
            max_uses: None,
            protected: None,
            require_check_in: None,
            check_in_interval: None,
            check_in_interval_count: None,
            max_users: None,
            scheme: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone)]
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
    pub machine_uniqueness_strategy: Option<UniquenessStrategy>,
    pub component_uniqueness_strategy: Option<UniquenessStrategy>,
    pub machine_matching_strategy: Option<MatchingStrategy>,
    pub component_matching_strategy: Option<MatchingStrategy>,
    pub expiration_strategy: ExpirationStrategy,
    pub expiration_basis: Option<String>,
    pub renewal_basis: Option<String>,
    pub authentication_strategy: AuthenticationStrategy,
    pub machine_leasing_strategy: LeasingStrategy,
    pub process_leasing_strategy: LeasingStrategy,
    pub overage_strategy: OverageStrategy,
    pub transfer_strategy: TransferStrategy,
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
    pub scheme: Option<Scheme>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListPoliciesOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
    pub product: Option<String>,
}

impl Policy {
    pub(crate) fn from(data: KeygenResponseData<PolicyAttributes>) -> Policy {
        Policy {
            id: data.id,
            name: data.attributes.name,
            duration: data.attributes.duration,
            strict: data.attributes.strict,
            floating: data.attributes.floating,
            require_heartbeat: data.attributes.require_heartbeat,
            heartbeat_duration: data.attributes.heartbeat_duration,
            heartbeat_cull_strategy: data.attributes.heartbeat_cull_strategy,
            heartbeat_resurrection_strategy: data.attributes.heartbeat_resurrection_strategy,
            heartbeat_basis: data.attributes.heartbeat_basis,
            machine_uniqueness_strategy: data.attributes.machine_uniqueness_strategy,
            component_uniqueness_strategy: data.attributes.component_uniqueness_strategy,
            machine_matching_strategy: data.attributes.machine_matching_strategy,
            component_matching_strategy: data.attributes.component_matching_strategy,
            expiration_strategy: data.attributes.expiration_strategy,
            expiration_basis: data.attributes.expiration_basis,
            renewal_basis: data.attributes.renewal_basis,
            authentication_strategy: data.attributes.authentication_strategy,
            machine_leasing_strategy: data.attributes.machine_leasing_strategy,
            process_leasing_strategy: data.attributes.process_leasing_strategy,
            overage_strategy: data.attributes.overage_strategy,
            transfer_strategy: data.attributes.transfer_strategy,
            max_machines: data.attributes.max_machines,
            max_processes: data.attributes.max_processes,
            max_cores: data.attributes.max_cores,
            max_uses: data.attributes.max_uses,
            encrypted: data.attributes.encrypted,
            protected: data.attributes.protected,
            require_check_in: data.attributes.require_check_in,
            check_in_interval: data.attributes.check_in_interval,
            check_in_interval_count: data.attributes.check_in_interval_count,
            use_pool: data.attributes.use_pool,
            max_licenses: data.attributes.max_licenses,
            max_users: data.attributes.max_users,
            scheme: data.attributes.scheme,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data.relationships.account.as_ref().and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
            product_id: data.relationships.product.as_ref().and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new policy
    pub async fn create(request: CreatePolicyRequest) -> Result<Policy, Error> {
        let client = Client::default();

        // Build attributes dynamically, only including non-None values
        let mut attributes = serde_json::Map::new();
        attributes.insert("name".to_string(), serde_json::Value::String(request.name));

        if let Some(duration) = request.duration {
            attributes.insert(
                "duration".to_string(),
                serde_json::Value::Number(duration.into()),
            );
        }
        if let Some(strict) = request.strict {
            attributes.insert("strict".to_string(), serde_json::Value::Bool(strict));
        }
        if let Some(floating) = request.floating {
            attributes.insert("floating".to_string(), serde_json::Value::Bool(floating));
        }
        if let Some(require_heartbeat) = request.require_heartbeat {
            attributes.insert(
                "requireHeartbeat".to_string(),
                serde_json::Value::Bool(require_heartbeat),
            );
        }
        if let Some(heartbeat_duration) = request.heartbeat_duration {
            attributes.insert(
                "heartbeatDuration".to_string(),
                serde_json::Value::Number(heartbeat_duration.into()),
            );
        }
        if let Some(ref heartbeat_cull_strategy) = request.heartbeat_cull_strategy {
            attributes.insert(
                "heartbeatCullStrategy".to_string(),
                serde_json::Value::String(heartbeat_cull_strategy.clone()),
            );
        }
        if let Some(ref heartbeat_resurrection_strategy) = request.heartbeat_resurrection_strategy {
            attributes.insert(
                "heartbeatResurrectionStrategy".to_string(),
                serde_json::Value::String(heartbeat_resurrection_strategy.clone()),
            );
        }
        if let Some(ref heartbeat_basis) = request.heartbeat_basis {
            attributes.insert(
                "heartbeatBasis".to_string(),
                serde_json::Value::String(heartbeat_basis.clone()),
            );
        }
        if let Some(machine_uniqueness_strategy) = request.machine_uniqueness_strategy {
            attributes.insert(
                "machineUniquenessStrategy".to_string(),
                serde_json::to_value(machine_uniqueness_strategy)?,
            );
        }
        if let Some(component_uniqueness_strategy) = request.component_uniqueness_strategy {
            attributes.insert(
                "componentUniquenessStrategy".to_string(),
                serde_json::to_value(component_uniqueness_strategy)?,
            );
        }
        if let Some(machine_matching_strategy) = request.machine_matching_strategy {
            attributes.insert(
                "machineMatchingStrategy".to_string(),
                serde_json::to_value(machine_matching_strategy)?,
            );
        }
        if let Some(component_matching_strategy) = request.component_matching_strategy {
            attributes.insert(
                "componentMatchingStrategy".to_string(),
                serde_json::to_value(component_matching_strategy)?,
            );
        }
        if let Some(expiration_strategy) = request.expiration_strategy {
            attributes.insert(
                "expirationStrategy".to_string(),
                serde_json::to_value(expiration_strategy)?,
            );
        }
        if let Some(ref expiration_basis) = request.expiration_basis {
            attributes.insert(
                "expirationBasis".to_string(),
                serde_json::Value::String(expiration_basis.clone()),
            );
        }
        if let Some(ref renewal_basis) = request.renewal_basis {
            attributes.insert(
                "renewalBasis".to_string(),
                serde_json::Value::String(renewal_basis.clone()),
            );
        }
        if let Some(authentication_strategy) = request.authentication_strategy {
            attributes.insert(
                "authenticationStrategy".to_string(),
                serde_json::to_value(authentication_strategy)?,
            );
        }
        if let Some(machine_leasing_strategy) = request.machine_leasing_strategy {
            attributes.insert(
                "machineLeasingStrategy".to_string(),
                serde_json::to_value(machine_leasing_strategy)?,
            );
        }
        if let Some(process_leasing_strategy) = request.process_leasing_strategy {
            attributes.insert(
                "processLeasingStrategy".to_string(),
                serde_json::to_value(process_leasing_strategy)?,
            );
        }
        if let Some(overage_strategy) = request.overage_strategy {
            attributes.insert(
                "overageStrategy".to_string(),
                serde_json::to_value(overage_strategy)?,
            );
        }
        if let Some(transfer_strategy) = request.transfer_strategy {
            attributes.insert(
                "transferStrategy".to_string(),
                serde_json::to_value(transfer_strategy)?,
            );
        }
        if let Some(max_machines) = request.max_machines {
            attributes.insert(
                "maxMachines".to_string(),
                serde_json::Value::Number(max_machines.into()),
            );
        }
        if let Some(max_processes) = request.max_processes {
            attributes.insert(
                "maxProcesses".to_string(),
                serde_json::Value::Number(max_processes.into()),
            );
        }
        if let Some(max_cores) = request.max_cores {
            attributes.insert(
                "maxCores".to_string(),
                serde_json::Value::Number(max_cores.into()),
            );
        }
        if let Some(max_uses) = request.max_uses {
            attributes.insert(
                "maxUses".to_string(),
                serde_json::Value::Number(max_uses.into()),
            );
        }
        if let Some(encrypted) = request.encrypted {
            attributes.insert("encrypted".to_string(), serde_json::Value::Bool(encrypted));
        }
        if let Some(protected) = request.protected {
            attributes.insert("protected".to_string(), serde_json::Value::Bool(protected));
        }
        if let Some(require_check_in) = request.require_check_in {
            attributes.insert(
                "requireCheckIn".to_string(),
                serde_json::Value::Bool(require_check_in),
            );
        }
        if let Some(ref check_in_interval) = request.check_in_interval {
            attributes.insert(
                "checkInInterval".to_string(),
                serde_json::Value::String(check_in_interval.clone()),
            );
        }
        if let Some(check_in_interval_count) = request.check_in_interval_count {
            attributes.insert(
                "checkInIntervalCount".to_string(),
                serde_json::Value::Number(check_in_interval_count.into()),
            );
        }
        if let Some(use_pool) = request.use_pool {
            attributes.insert("usePool".to_string(), serde_json::Value::Bool(use_pool));
        }
        if let Some(max_licenses) = request.max_licenses {
            attributes.insert(
                "maxLicenses".to_string(),
                serde_json::Value::Number(max_licenses.into()),
            );
        }
        if let Some(max_users) = request.max_users {
            attributes.insert(
                "maxUsers".to_string(),
                serde_json::Value::Number(max_users.into()),
            );
        }
        if let Some(scheme) = request.scheme {
            attributes.insert("scheme".to_string(), serde_json::to_value(scheme)?);
        }
        if let Some(ref metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "policies",
                "attributes": attributes,
                "relationships": {
                    "product": {
                        "data": {
                            "type": "products",
                            "id": request.product_id
                        }
                    }
                }
            }
        });

        let response = client.post("policies", Some(&body), None::<&()>).await?;
        let policy_response: PolicyResponse = serde_json::from_value(response.body)?;
        Ok(Policy::from(policy_response.data))
    }

    /// List policies with optional pagination and filtering
    pub async fn list(options: Option<ListPoliciesOptions>) -> Result<Vec<Policy>, Error> {
        let client = Client::default();
        let response = client.get("policies", options.as_ref()).await?;
        let policies_response: PoliciesResponse = serde_json::from_value(response.body)?;
        Ok(policies_response
            .data
            .into_iter()
            .map(Policy::from)
            .collect())
    }

    /// Get a policy by ID
    pub async fn get(id: &str) -> Result<Policy, Error> {
        let client = Client::default();
        let endpoint = format!("policies/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let policy_response: PolicyResponse = serde_json::from_value(response.body)?;
        Ok(Policy::from(policy_response.data))
    }

    /// Update a policy
    pub async fn update(&self, request: UpdatePolicyRequest) -> Result<Policy, Error> {
        let client = Client::default();
        let endpoint = format!("policies/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(duration) = request.duration {
            attributes.insert(
                "duration".to_string(),
                serde_json::Value::Number(duration.into()),
            );
        }
        if let Some(strict) = request.strict {
            attributes.insert("strict".to_string(), serde_json::Value::Bool(strict));
        }
        if let Some(floating) = request.floating {
            attributes.insert("floating".to_string(), serde_json::Value::Bool(floating));
        }
        if let Some(require_heartbeat) = request.require_heartbeat {
            attributes.insert(
                "requireHeartbeat".to_string(),
                serde_json::Value::Bool(require_heartbeat),
            );
        }
        if let Some(heartbeat_duration) = request.heartbeat_duration {
            attributes.insert(
                "heartbeatDuration".to_string(),
                serde_json::Value::Number(heartbeat_duration.into()),
            );
        }
        if let Some(expiration_strategy) = request.expiration_strategy {
            attributes.insert(
                "expirationStrategy".to_string(),
                serde_json::to_value(expiration_strategy)?,
            );
        }
        if let Some(authentication_strategy) = request.authentication_strategy {
            attributes.insert(
                "authenticationStrategy".to_string(),
                serde_json::to_value(authentication_strategy)?,
            );
        }
        if let Some(machine_leasing_strategy) = request.machine_leasing_strategy {
            attributes.insert(
                "machineLeasingStrategy".to_string(),
                serde_json::to_value(machine_leasing_strategy)?,
            );
        }
        if let Some(process_leasing_strategy) = request.process_leasing_strategy {
            attributes.insert(
                "processLeasingStrategy".to_string(),
                serde_json::to_value(process_leasing_strategy)?,
            );
        }
        if let Some(overage_strategy) = request.overage_strategy {
            attributes.insert(
                "overageStrategy".to_string(),
                serde_json::to_value(overage_strategy)?,
            );
        }
        if let Some(transfer_strategy) = request.transfer_strategy {
            attributes.insert(
                "transferStrategy".to_string(),
                serde_json::to_value(transfer_strategy)?,
            );
        }
        if let Some(max_machines) = request.max_machines {
            attributes.insert(
                "maxMachines".to_string(),
                serde_json::Value::Number(max_machines.into()),
            );
        }
        if let Some(max_processes) = request.max_processes {
            attributes.insert(
                "maxProcesses".to_string(),
                serde_json::Value::Number(max_processes.into()),
            );
        }
        if let Some(max_cores) = request.max_cores {
            attributes.insert(
                "maxCores".to_string(),
                serde_json::Value::Number(max_cores.into()),
            );
        }
        if let Some(max_uses) = request.max_uses {
            attributes.insert(
                "maxUses".to_string(),
                serde_json::Value::Number(max_uses.into()),
            );
        }
        if let Some(protected) = request.protected {
            attributes.insert("protected".to_string(), serde_json::Value::Bool(protected));
        }
        if let Some(require_check_in) = request.require_check_in {
            attributes.insert(
                "requireCheckIn".to_string(),
                serde_json::Value::Bool(require_check_in),
            );
        }
        if let Some(check_in_interval) = request.check_in_interval {
            attributes.insert(
                "checkInInterval".to_string(),
                serde_json::Value::String(check_in_interval),
            );
        }
        if let Some(check_in_interval_count) = request.check_in_interval_count {
            attributes.insert(
                "checkInIntervalCount".to_string(),
                serde_json::Value::Number(check_in_interval_count.into()),
            );
        }
        if let Some(max_users) = request.max_users {
            attributes.insert(
                "maxUsers".to_string(),
                serde_json::Value::Number(max_users.into()),
            );
        }
        if let Some(scheme) = request.scheme {
            attributes.insert("scheme".to_string(), serde_json::to_value(scheme)?);
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "policies",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let policy_response: PolicyResponse = serde_json::from_value(response.body)?;
        Ok(Policy::from(policy_response.data))
    }

    /// Delete a policy
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default();
        let endpoint = format!("policies/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Pop a key from policy pool
    pub async fn pop_key(&self) -> Result<String, Error> {
        let client = Client::default();
        let endpoint = format!("policies/{}/pool", self.id);
        let response: crate::client::Response<serde_json::Value> =
            client.delete(&endpoint, None::<&()>).await?;

        // Extract key from response
        let key_data = response.body["data"]["attributes"]["key"]
            .as_str()
            .ok_or_else(|| Error::KeygenApiError {
                code: "INVALID_RESPONSE".to_string(),
                detail: "Invalid key response format".to_string(),
                body: response.body.clone(),
            })?;

        Ok(key_data.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheme_serialization() {
        // Test Ed25519 scheme
        let scheme = Scheme::Ed25519Sign;
        let json = serde_json::to_string(&scheme).unwrap();
        assert_eq!(json, "\"ED25519_SIGN\"");

        // Test RSA schemes
        let scheme = Scheme::Rsa2048Pkcs1PssSignV2;
        let json = serde_json::to_string(&scheme).unwrap();
        assert_eq!(json, "\"RSA_2048_PKCS1_PSS_SIGN_V2\"");

        let scheme = Scheme::Rsa2048JwtRs256;
        let json = serde_json::to_string(&scheme).unwrap();
        assert_eq!(json, "\"RSA_2048_JWT_RS256\"");

        // Test legacy encrypt scheme
        let scheme = Scheme::LegacyEncrypt;
        let json = serde_json::to_string(&scheme).unwrap();
        assert_eq!(json, "\"LEGACY_ENCRYPT\"");
    }

    #[test]
    fn test_scheme_deserialization() {
        // Test deserializing from JSON
        let json = "\"ED25519_SIGN\"";
        let scheme: Scheme = serde_json::from_str(json).unwrap();
        assert_eq!(scheme, Scheme::Ed25519Sign);

        let json = "\"RSA_2048_PKCS1_PSS_SIGN_V2\"";
        let scheme: Scheme = serde_json::from_str(json).unwrap();
        assert_eq!(scheme, Scheme::Rsa2048Pkcs1PssSignV2);

        // Test legacy encrypt scheme
        let json = "\"LEGACY_ENCRYPT\"";
        let scheme: Scheme = serde_json::from_str(json).unwrap();
        assert_eq!(scheme, Scheme::LegacyEncrypt);
    }

    #[test]
    fn test_deprecated_schemes() {
        // Test deprecated schemes still work
        let json = "\"RSA_2048_PKCS1_PSS_SIGN\"";
        let scheme: Scheme = serde_json::from_str(json).unwrap();
        assert_eq!(scheme, Scheme::Rsa2048Pkcs1PssSign);

        let json = "\"RSA_2048_PKCS1_SIGN\"";
        let scheme: Scheme = serde_json::from_str(json).unwrap();
        assert_eq!(scheme, Scheme::Rsa2048Pkcs1Sign);
    }

    #[test]
    fn test_policy_relationships() {
        use crate::{KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData};
        
        // Test that account_id and product_id are properly extracted from relationships
        let policy_data = KeygenResponseData {
            id: "test-policy-id".to_string(),
            r#type: "policies".to_string(),
            attributes: PolicyAttributes {
                name: "Test Policy".to_string(),
                duration: Some(3600),
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
                max_machines: Some(5),
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
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships {
                policy: None,
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                product: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    }),
                    links: None,
                }),
                group: None,
                owner: None,
                users: None,
                machines: None,
                environment: None,
                license: None,
            },
        };

        let policy = Policy::from(policy_data);
        
        assert_eq!(policy.account_id, Some("test-account-id".to_string()));
        assert_eq!(policy.product_id, Some("test-product-id".to_string()));
    }
}
