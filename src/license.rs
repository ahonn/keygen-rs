use std::env;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::certificate::CertificateFileResponse;
use crate::client::Client;
use crate::component::Component;
use crate::config::get_config;
use crate::entitlement::{Entitlement, EntitlementsResponse};
use crate::errors::Error;
use crate::license_file::LicenseFile;
use crate::machine::{Machine, MachineResponse, MachinesResponse};
use crate::verifier::Verifier;
use crate::KeygenResponseData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemeCode {
    #[serde(rename = "ED25519_SIGN")]
    Ed25519Sign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LicenseResponse<M> {
    pub meta: Option<M>,
    pub data: KeygenResponseData<LicenseAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValidationMeta {
    pub ts: DateTime<Utc>,
    pub valid: bool,
    pub detail: String,
    pub code: String,
    pub scope: ValidationScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ValidationScope {
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LicenseAttributes {
    pub key: String,
    pub name: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub uses: Option<i32>,
    #[serde(rename = "maxMachines")]
    pub max_machines: Option<i32>,
    #[serde(rename = "maxCores")]
    pub max_cores: Option<i32>,
    #[serde(rename = "maxUses")]
    pub max_uses: Option<i32>,
    #[serde(rename = "maxProcesses")]
    pub max_processes: Option<i32>,
    #[serde(rename = "maxUsers")]
    pub max_users: Option<i32>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    #[serde(skip_serializing)]
    pub scheme: Option<SchemeCode>,
    pub key: String,
    pub name: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
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
    pub metadata: HashMap<String, Value>,
    pub account_id: Option<String>,
    pub product_id: Option<String>,
    pub group_id: Option<String>,
    pub owner_id: Option<String>,
}

pub struct LicenseCheckoutOpts {
    pub ttl: Option<i64>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Default, Serialize)]
pub struct PaginationOptions {
    pub limit: Option<i32>,           // Number of resources to return (1-100, default 10)
    pub page_number: Option<i32>,     // Page number to retrieve
    pub page_size: Option<i32>,       // Number of resources per page (1-100)
}

/// Simple license list options with common filters
#[derive(Debug, Default)]
pub struct LicenseListOptions {
    // Pagination - following Keygen API standards
    pub limit: Option<i32>,           // Number of resources to return (1-100, default 10)
    pub page_number: Option<i32>,     // Page number to retrieve
    pub page_size: Option<i32>,       // Number of resources per page (1-100)

    // Common filters
    pub status: Option<String>,  // "ACTIVE", "EXPIRED", "SUSPENDED", etc.
    pub product: Option<String>, // Product ID
    pub policy: Option<String>,  // Policy ID
    pub owner: Option<String>,   // Owner ID or email
    pub user: Option<String>,    // User ID or email
}

/// Request structure for creating a new license with complete API support
#[derive(Debug, Default)]
pub struct LicenseCreateRequest {
    // Required relationship
    pub policy_id: String,

    // Optional attributes
    pub name: Option<String>,
    pub key: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub max_machines: Option<i32>,
    pub max_processes: Option<i32>,
    pub max_users: Option<i32>,
    pub max_cores: Option<i32>,
    pub max_uses: Option<i32>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,

    // Optional relationships
    pub owner_id: Option<String>, // User ID
    pub group_id: Option<String>, // Group ID
}

/// Request structure for updating a license with complete API support
#[derive(Debug, Default)]
pub struct LicenseUpdateRequest {
    // All optional attributes that can be updated
    pub name: Option<String>,
    pub expiry: Option<DateTime<Utc>>,
    pub max_machines: Option<Option<i32>>, // None = don't update, Some(None) = set to null, Some(Some(val)) = set to val
    pub max_processes: Option<Option<i32>>,
    pub max_users: Option<Option<i32>>,
    pub max_cores: Option<Option<i32>>,
    pub max_uses: Option<Option<i32>>,
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl LicenseCreateRequest {
    /// Create a new license creation request with the required policy ID
    pub fn new(policy_id: String) -> Self {
        Self {
            policy_id,
            ..Default::default()
        }
    }

    /// Set the license name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set a custom license key
    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    /// Set the expiry date
    pub fn with_expiry(mut self, expiry: DateTime<Utc>) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Set the maximum number of machines
    pub fn with_max_machines(mut self, max_machines: i32) -> Self {
        self.max_machines = Some(max_machines);
        self
    }

    /// Set the maximum number of processes
    pub fn with_max_processes(mut self, max_processes: i32) -> Self {
        self.max_processes = Some(max_processes);
        self
    }

    /// Set the maximum number of users
    pub fn with_max_users(mut self, max_users: i32) -> Self {
        self.max_users = Some(max_users);
        self
    }

    /// Set the maximum number of cores
    pub fn with_max_cores(mut self, max_cores: i32) -> Self {
        self.max_cores = Some(max_cores);
        self
    }

    /// Set the maximum number of uses
    pub fn with_max_uses(mut self, max_uses: i32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Set the protected flag
    pub fn with_protected(mut self, protected: bool) -> Self {
        self.protected = Some(protected);
        self
    }

    /// Set the suspended flag
    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.suspended = Some(suspended);
        self
    }

    /// Set the permissions array
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set the metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set the owner (user) ID
    pub fn with_owner_id(mut self, owner_id: String) -> Self {
        self.owner_id = Some(owner_id);
        self
    }

    /// Set the group ID
    pub fn with_group_id(mut self, group_id: String) -> Self {
        self.group_id = Some(group_id);
        self
    }

    /// Convert this request to attributes and relationships JSON maps for the API
    pub fn to_json_body(self) -> Value {
        let mut attributes = serde_json::Map::new();
        let mut relationships = serde_json::Map::new();

        // Build attributes
        if let Some(name) = self.name {
            attributes.insert("name".to_string(), json!(name));
        }
        if let Some(key) = self.key {
            attributes.insert("key".to_string(), json!(key));
        }
        if let Some(expiry) = self.expiry {
            attributes.insert("expiry".to_string(), json!(expiry));
        }
        if let Some(max_machines) = self.max_machines {
            attributes.insert("maxMachines".to_string(), json!(max_machines));
        }
        if let Some(max_processes) = self.max_processes {
            attributes.insert("maxProcesses".to_string(), json!(max_processes));
        }
        if let Some(max_users) = self.max_users {
            attributes.insert("maxUsers".to_string(), json!(max_users));
        }
        if let Some(max_cores) = self.max_cores {
            attributes.insert("maxCores".to_string(), json!(max_cores));
        }
        if let Some(max_uses) = self.max_uses {
            attributes.insert("maxUses".to_string(), json!(max_uses));
        }
        if let Some(protected) = self.protected {
            attributes.insert("protected".to_string(), json!(protected));
        }
        if let Some(suspended) = self.suspended {
            attributes.insert("suspended".to_string(), json!(suspended));
        }
        if let Some(permissions) = self.permissions {
            attributes.insert("permissions".to_string(), json!(permissions));
        }
        if let Some(metadata) = self.metadata {
            attributes.insert("metadata".to_string(), json!(metadata));
        }

        // Build relationships - policy is required
        relationships.insert(
            "policy".to_string(),
            json!({
                "data": {
                    "type": "policies",
                    "id": self.policy_id
                }
            }),
        );

        if let Some(owner_id) = self.owner_id {
            relationships.insert(
                "owner".to_string(),
                json!({
                    "data": {
                        "type": "users",
                        "id": owner_id
                    }
                }),
            );
        }

        if let Some(group_id) = self.group_id {
            relationships.insert(
                "group".to_string(),
                json!({
                    "data": {
                        "type": "groups",
                        "id": group_id
                    }
                }),
            );
        }

        json!({
            "data": {
                "type": "licenses",
                "attributes": attributes,
                "relationships": relationships
            }
        })
    }
}

impl LicenseUpdateRequest {
    /// Create a new empty license update request
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the license name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the expiry date
    pub fn with_expiry(mut self, expiry: DateTime<Utc>) -> Self {
        self.expiry = Some(expiry);
        self
    }

    /// Set the maximum number of machines
    pub fn with_max_machines(mut self, max_machines: i32) -> Self {
        self.max_machines = Some(Some(max_machines));
        self
    }

    /// Clear the maximum number of machines (set to null)
    pub fn clear_max_machines(mut self) -> Self {
        self.max_machines = Some(None);
        self
    }

    /// Set the maximum number of processes
    pub fn with_max_processes(mut self, max_processes: i32) -> Self {
        self.max_processes = Some(Some(max_processes));
        self
    }

    /// Clear the maximum number of processes (set to null)
    pub fn clear_max_processes(mut self) -> Self {
        self.max_processes = Some(None);
        self
    }

    /// Set the maximum number of users
    pub fn with_max_users(mut self, max_users: i32) -> Self {
        self.max_users = Some(Some(max_users));
        self
    }

    /// Clear the maximum number of users (set to null)
    pub fn clear_max_users(mut self) -> Self {
        self.max_users = Some(None);
        self
    }

    /// Set the maximum number of cores
    pub fn with_max_cores(mut self, max_cores: i32) -> Self {
        self.max_cores = Some(Some(max_cores));
        self
    }

    /// Clear the maximum number of cores (set to null)
    pub fn clear_max_cores(mut self) -> Self {
        self.max_cores = Some(None);
        self
    }

    /// Set the maximum number of uses
    pub fn with_max_uses(mut self, max_uses: i32) -> Self {
        self.max_uses = Some(Some(max_uses));
        self
    }

    /// Clear the maximum number of uses (set to null)
    pub fn clear_max_uses(mut self) -> Self {
        self.max_uses = Some(None);
        self
    }

    /// Set the protected flag
    pub fn with_protected(mut self, protected: bool) -> Self {
        self.protected = Some(protected);
        self
    }

    /// Set the suspended flag
    pub fn with_suspended(mut self, suspended: bool) -> Self {
        self.suspended = Some(suspended);
        self
    }

    /// Set the permissions array
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = Some(permissions);
        self
    }

    /// Set the metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Convert this request to complete JSON body for the API
    pub fn to_json_body(self) -> Value {
        let mut attributes = serde_json::Map::new();

        if let Some(name) = self.name {
            attributes.insert("name".to_string(), json!(name));
        }
        if let Some(expiry) = self.expiry {
            attributes.insert("expiry".to_string(), json!(expiry));
        }
        if let Some(max_machines) = self.max_machines {
            attributes.insert("maxMachines".to_string(), json!(max_machines));
        }
        if let Some(max_processes) = self.max_processes {
            attributes.insert("maxProcesses".to_string(), json!(max_processes));
        }
        if let Some(max_users) = self.max_users {
            attributes.insert("maxUsers".to_string(), json!(max_users));
        }
        if let Some(max_cores) = self.max_cores {
            attributes.insert("maxCores".to_string(), json!(max_cores));
        }
        if let Some(max_uses) = self.max_uses {
            attributes.insert("maxUses".to_string(), json!(max_uses));
        }
        if let Some(protected) = self.protected {
            attributes.insert("protected".to_string(), json!(protected));
        }
        if let Some(suspended) = self.suspended {
            attributes.insert("suspended".to_string(), json!(suspended));
        }
        if let Some(permissions) = self.permissions {
            attributes.insert("permissions".to_string(), json!(permissions));
        }
        if let Some(metadata) = self.metadata {
            attributes.insert("metadata".to_string(), json!(metadata));
        }

        json!({
            "data": {
                "type": "licenses",
                "attributes": attributes
            }
        })
    }
}

impl License {
    pub(crate) fn from(data: KeygenResponseData<LicenseAttributes>) -> License {
        License {
            id: data.id,
            scheme: None,
            key: data.attributes.key,
            name: data.attributes.name,
            expiry: data.attributes.expiry,
            status: data.attributes.status,
            uses: data.attributes.uses,
            max_machines: data.attributes.max_machines,
            max_cores: data.attributes.max_cores,
            max_uses: data.attributes.max_uses,
            max_processes: data.attributes.max_processes,
            max_users: data.attributes.max_users,
            protected: data.attributes.protected,
            suspended: data.attributes.suspended,
            permissions: data.attributes.permissions,
            policy: data
                .relationships
                .policy
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            metadata: data.attributes.metadata,
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
            product_id: data
                .relationships
                .product
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            group_id: data
                .relationships
                .group
                .as_ref()
                .and_then(|g| g.data.as_ref().map(|d| d.id.clone())),
            owner_id: data
                .relationships
                .owner
                .as_ref()
                .and_then(|o| o.data.as_ref().map(|d| d.id.clone())),
        }
    }

    pub(crate) fn from_signed_key(scheme: SchemeCode, signed_key: &str) -> License {
        License {
            id: String::new(),
            scheme: Some(scheme),
            key: signed_key.to_string(),
            name: None,
            expiry: None,
            status: None,
            uses: None,
            max_machines: None,
            max_cores: None,
            max_uses: None,
            max_processes: None,
            max_users: None,
            protected: None,
            suspended: None,
            permissions: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
        }
    }

    fn build_scope(fingerprints: &[String], entitlements: &[String]) -> Result<Value, Error> {
        let config = get_config()?;
        let mut scope = json!({
            "product": config.product.to_string(),
        });

        if !fingerprints.is_empty() {
            scope["fingerprint"] = json!(fingerprints[0]);
            if fingerprints.len() > 1 {
                scope["components"] = json!(fingerprints[1..].to_vec());
            }
        }

        if !entitlements.is_empty() {
            scope["entitlements"] = json!(entitlements);
        }

        if let Some(env) = config.environment.as_ref() {
            scope["environment"] = json!(env);
        }

        Ok(scope)
    }

    pub async fn validate(
        self,
        fingerprints: &[String],
        entitlements: &[String],
    ) -> Result<License, Error> {
        let client = Client::default()?;
        let scope = Self::build_scope(fingerprints, entitlements)?;
        let params = json!({
            "meta": {
                "nonce": chrono::Utc::now().timestamp(),
                "scope": scope
            }
        });

        let response = client
            .post(
                &format!("licenses/{}/actions/validate", self.id),
                Some(&params),
                None::<&()>,
            )
            .await?;
        let validation: LicenseResponse<ValidationMeta> = serde_json::from_value(response.body)?;
        let meta = validation.meta.clone().unwrap();
        if !meta.valid {
            return Err(self.handle_validation_code(&meta));
        };
        let license = License::from(validation.data);
        Ok(license)
    }

    pub async fn validate_key(
        self,
        fingerprints: &[String],
        entitlements: &[String],
    ) -> Result<License, Error> {
        let client = Client::default()?;
        let scope = Self::build_scope(fingerprints, entitlements)?;
        let params = json!({
            "meta": {
                "key": self.key.clone(),
                "scope": scope
            }
        });

        let response = client
            .post(&"licenses/actions/validate-key", Some(&params), None::<&()>)
            .await?;
        let validation: LicenseResponse<ValidationMeta> = serde_json::from_value(response.body)?;
        let meta = validation.meta.clone().unwrap();
        if !meta.valid {
            return Err(self.handle_validation_code(&meta));
        };
        let license = License::from(validation.data);
        Ok(license)
    }

    pub fn verify(&self) -> Result<Vec<u8>, Error> {
        if self.scheme.is_none() {
            return Err(Error::LicenseNotSigned);
        }
        let config = get_config()?;
        if let Some(public_key) = config.public_key {
            let verifier = Verifier::new(public_key);
            verifier.verify_license(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    pub async fn activate(
        &self,
        fingerprint: &str,
        components: &[Component],
    ) -> Result<Machine, Error> {
        let config = get_config()?;
        let client = Client::default()?;
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from("unknown"));
        let platform = config
            .platform
            .or_else(|| Some(format!("{}/{}", env::consts::OS, env::consts::ARCH)));

        let mut params = json!({
          "data": {
            "type": "machines",
            "attributes": {
              "fingerprint": fingerprint,
              "cores": num_cpus::get(),
              "hostname": hostname,
              "platform": platform,
            },
            "relationships": {
              "license": {
                "data": {
                  "type": "licenses",
                  "id": self.id
                }
              },
            }
          }
        });
        if components.len() > 0 {
            params["data"]["relationships"]["components"] = json!({
                "data": components
                    .iter()
                    .map(|comp| json!({
                        "type": "components",
                        "attributes": {
                            "fingerprint": comp.fingerprint,
                            "name": comp.name
                        }
                    }))
                    .collect::<Vec<serde_json::Value>>()
            });
        }

        let response = client.post("machines", Some(&params), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        let machine = Machine::from(machine_response.data);
        Ok(machine)
    }

    pub async fn deactivate(&self, id: &str) -> Result<(), Error> {
        let client = Client::default()?;
        let _response = client
            .delete::<(), serde_json::Value>(&format!("machines/{}", id), None::<&()>)
            .await?;
        Ok(())
    }

    pub async fn machine(&self, id: &str) -> Result<Machine, Error> {
        let client = Client::default()?;
        let response = client.get(&format!("machines/{}", id), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        let machine = Machine::from(machine_response.data);
        Ok(machine)
    }

    pub async fn machines(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<Machine>, Error> {
        let client = Client::default()?;
        let mut query = json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            } else {
                query["limit"] = json!(100);
            }

            if let Some(page_number) = opts.page_number {
                query["page[number]"] = json!(page_number);
            }

            if let Some(page_size) = opts.page_size {
                query["page[size]"] = json!(page_size);
            }
        } else {
            query["limit"] = json!(100);
        }

        let response = client
            .get(&format!("licenses/{}/machines", self.id), Some(&query))
            .await?;
        let machines_response: MachinesResponse = serde_json::from_value(response.body)?;
        let machines = machines_response
            .data
            .iter()
            .map(|d| Machine::from(d.clone()))
            .collect();
        Ok(machines)
    }

    pub async fn entitlements(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<Entitlement>, Error> {
        let client = Client::default()?;
        let mut query = json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            } else {
                query["limit"] = json!(100);
            }

            if let Some(page_number) = opts.page_number {
                query["page[number]"] = json!(page_number);
            }

            if let Some(page_size) = opts.page_size {
                query["page[size]"] = json!(page_size);
            }
        } else {
            query["limit"] = json!(100);
        }

        let response = client
            .get(&format!("licenses/{}/entitlements", self.id), Some(&query))
            .await?;
        let entitlements_response: EntitlementsResponse = serde_json::from_value(response.body)?;
        let entitlements = entitlements_response
            .data
            .iter()
            .map(|d| Entitlement::from(d.clone()))
            .collect();
        Ok(entitlements)
    }

    pub async fn checkout(&self, options: &LicenseCheckoutOpts) -> Result<LicenseFile, Error> {
        let client = Client::default()?;
        let mut query = json!({
            "encrypt": 1,
            "include": "entitlements"
        });

        if let Some(ttl) = options.ttl {
            query["ttl"] = ttl.into();
        }

        if let Some(ref include) = options.include {
            query["include"] = json!(include.join(","));
        }

        let response = client
            .post(
                &format!("licenses/{}/actions/check-out", self.id),
                None::<&()>,
                Some(&query),
            )
            .await?;
        let license_file_response: CertificateFileResponse = serde_json::from_value(response.body)?;
        let license_file = LicenseFile::from(license_file_response.data);
        Ok(license_file)
    }

    fn handle_validation_code(&self, meta: &ValidationMeta) -> Error {
        let code = meta.code.clone();
        let detail = meta.detail.clone();
        match code.as_str() {
            "FINGERPRINT_SCOPE_MISMATCH" | "NO_MACHINES" | "NO_MACHINE" => {
                Error::LicenseNotActivated {
                    code,
                    detail,
                    license: self.clone(),
                }
            }
            "EXPIRED" => Error::LicenseExpired { code, detail },
            "SUSPENDED" => Error::LicenseSuspended { code, detail },
            "TOO_MANY_MACHINES" => Error::LicenseTooManyMachines { code, detail },
            "TOO_MANY_CORES" => Error::LicenseTooManyCores { code, detail },
            "TOO_MANY_PROCESSES" => Error::LicenseTooManyProcesses { code, detail },
            "FINGERPRINT_SCOPE_REQUIRED" | "FINGERPRINT_SCOPE_EMPTY" => {
                Error::ValidationFingerprintMissing { code, detail }
            }
            "COMPONENTS_SCOPE_REQUIRED" | "COMPONENTS_SCOPE_EMPTY" => {
                Error::ValidationComponentsMissing { code, detail }
            }
            "COMPONENTS_SCOPE_MISMATCH" => Error::ComponentNotActivated { code, detail },
            "HEARTBEAT_NOT_STARTED" => Error::HeartbeatRequired { code, detail },
            "HEARTBEAT_DEAD" => Error::HeartbeatDead { code, detail },
            "PRODUCT_SCOPE_REQUIRED" | "PRODUCT_SCOPE_EMPTY" => {
                Error::ValidationProductMissing { code, detail }
            }
            _ => Error::LicenseKeyInvalid { code, detail },
        }
    }

    /// Create a new license using the comprehensive request structure
    #[cfg(feature = "token")]
    pub async fn create(request: LicenseCreateRequest) -> Result<License, Error> {
        let client = Client::default()?;
        let body = request.to_json_body();
        let response = client.post("licenses", Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// List all licenses with optional filtering
    #[cfg(feature = "token")]
    pub async fn list(options: Option<&LicenseListOptions>) -> Result<Vec<License>, Error> {
        let client = Client::default()?;
        let mut query = json!({});

        if let Some(opts) = options {
            // Pagination - following Keygen API standards
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            }
            if let Some(page_number) = opts.page_number {
                query["page[number]"] = json!(page_number);
            }
            if let Some(page_size) = opts.page_size {
                query["page[size]"] = json!(page_size);
            }

            // Simple filters
            if let Some(ref status) = opts.status {
                query["status"] = json!(status);
            }
            if let Some(ref product) = opts.product {
                query["product"] = json!(product);
            }
            if let Some(ref policy) = opts.policy {
                query["policy"] = json!(policy);
            }
            if let Some(ref owner) = opts.owner {
                query["owner"] = json!(owner);
            }
            if let Some(ref user) = opts.user {
                query["user"] = json!(user);
            }
        }

        let response = client.get("licenses", Some(&query)).await?;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct LicensesResponse {
            pub data: Vec<KeygenResponseData<LicenseAttributes>>,
        }

        let licenses_response: LicensesResponse = serde_json::from_value(response.body)?;
        Ok(licenses_response
            .data
            .into_iter()
            .map(License::from)
            .collect())
    }

    /// Get a license by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<License, Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Update a license
    #[cfg(feature = "token")]
    pub async fn update(&self, request: LicenseUpdateRequest) -> Result<License, Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}", self.id);
        let body = request.to_json_body();
        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Delete a license
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Suspend a license
    #[cfg(feature = "token")]
    pub async fn suspend(&self) -> Result<License, Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}/actions/suspend", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Reinstate a suspended license
    #[cfg(feature = "token")]
    pub async fn reinstate(&self) -> Result<License, Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}/actions/reinstate", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Renew a license
    #[cfg(feature = "token")]
    pub async fn renew(&self) -> Result<License, Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}/actions/renew", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Revoke a license
    #[cfg(feature = "token")]
    pub async fn revoke(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}/actions/revoke", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Attach entitlements to a license
    #[cfg(feature = "token")]
    pub async fn attach_entitlements(&self, entitlement_ids: &[String]) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}/entitlements", self.id);
        
        let data: Vec<Value> = entitlement_ids
            .iter()
            .map(|id| json!({
                "type": "entitlements",
                "id": id
            }))
            .collect();

        let body = json!({
            "data": data
        });

        client.post::<Value, Value, ()>(&endpoint, Some(&body), None::<&()>).await?;
        Ok(())
    }

    /// Detach entitlements from a license
    #[cfg(feature = "token")]
    pub async fn detach_entitlements(&self, entitlement_ids: &[String]) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("licenses/{}/entitlements", self.id);
        
        let data: Vec<Value> = entitlement_ids
            .iter()
            .map(|id| json!({
                "type": "entitlements",
                "id": id
            }))
            .collect();

        let body = json!({
            "data": data
        });

        client.delete::<Value, Value>(&endpoint, Some(&body)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{reset_config, set_config, KeygenConfig};
    use chrono::TimeZone;
    use mockito::{mock, server_url};
    use serde_json::json;

    fn create_test_license() -> License {
        License {
            id: "test_license_id".to_string(),
            scheme: None,
            name: Some("Test License".to_string()),
            key: "TEST-LICENSE-KEY".to_string(),
            expiry: None,
            status: None,
            uses: None,
            max_machines: None,
            max_cores: None,
            max_uses: None,
            max_processes: None,
            max_users: None,
            protected: None,
            suspended: None,
            permissions: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
        }
    }

    fn get_mock_body() -> String {
        json!({
            "meta": {
                "ts": "2021-01-01T00:00:00Z",
                "valid": true,
                "detail": "is valid",
                "code": "VALID",
                "scope": {
                    "fingerprint": "test_fingerprint",
                    "components": ["comp1", "comp2"],
                    "product": "test_product"
                }
            },
            "data": {
                "id": "test_license_id",
                "type": "licenses",
                "attributes": {
                    "name": "Test License",
                    "key": "TEST-LICENSE-KEY",
                    "expiry": null,
                    "status": "valid",
                    "metadata": {
                        "customer_name": "Test Customer",
                        "customer_email": "test@example.com",
                        "is_premium": true
                    }
                },
                "relationships": {
                    "policy": {
                        "data": {
                            "type": "policies",
                            "id": "11314277-0f31-4a77-9366-0299e9f52123"
                        }
                    }
                }
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn test_validate() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let result = license
            .validate(
                &[
                    "test_fingerprint".to_string(),
                    "comp1".to_string(),
                    "comp2".to_string(),
                ],
                &[],
            )
            .await;
        assert!(result.is_ok());
        reset_config();
    }

    #[tokio::test]
    async fn test_validate_key() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/actions/validate-key")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            license_key: Some("TEST-LICENSE-KEY".to_string()),
            ..Default::default()
        });

        let result = license
            .validate_key(
                &[
                    "test_fingerprint".to_string(),
                    "comp1".to_string(),
                    "comp2".to_string(),
                ],
                &[],
            )
            .await;
        assert!(result.is_ok());
        reset_config();
    }

    #[test]
    fn test_verify() {
        let mut license = create_test_license();

        license.scheme = Some(SchemeCode::Ed25519Sign);
        let result = license.verify();
        assert!(matches!(result, Err(Error::PublicKeyMissing)));

        license.scheme = None;
        let result = license.verify();
        assert!(matches!(result, Err(Error::LicenseNotSigned)));
    }

    #[tokio::test]
    async fn test_validate_with_metadata() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let result = license
            .validate(
                &[
                    "test_fingerprint".to_string(),
                    "comp1".to_string(),
                    "comp2".to_string(),
                ],
                &[],
            )
            .await;

        assert!(result.is_ok());
        let validated_license = result.unwrap();

        // Verify metadata fields
        assert!(validated_license.metadata.contains_key("customer_name"));
        assert_eq!(
            validated_license
                .metadata
                .get("customer_name")
                .unwrap()
                .as_str()
                .unwrap(),
            "Test Customer"
        );

        assert!(validated_license.metadata.contains_key("customer_email"));
        assert_eq!(
            validated_license
                .metadata
                .get("customer_email")
                .unwrap()
                .as_str()
                .unwrap(),
            "test@example.com"
        );

        assert!(validated_license.metadata.contains_key("is_premium"));
        assert!(validated_license
            .metadata
            .get("is_premium")
            .unwrap()
            .as_bool()
            .unwrap());

        reset_config();
    }

    #[tokio::test]
    async fn test_pagination_options() {
        let license = create_test_license();
        let _m = mock("GET", "/v1/licenses/test_license_id/machines")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "50".into()),
                mockito::Matcher::UrlEncoded("page[number]".into(), "2".into()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "10".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": []
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let pagination_options = PaginationOptions {
            limit: Some(50),
            page_number: Some(2),
            page_size: Some(10),
        };

        let result = license.machines(Some(&pagination_options)).await;
        assert!(result.is_ok());
        reset_config();
    }

    #[tokio::test]
    async fn test_validation_errors() {
        let license = create_test_license();

        // Test expired license
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "meta": {
                        "ts": "2021-01-01T00:00:00Z",
                        "valid": false,
                        "detail": "license expired",
                        "code": "EXPIRED",
                        "scope": {
                            "fingerprint": "test_fingerprint"
                        }
                    },
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": null,
                            "status": "expired",
                            "uses": null,
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": null,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy_123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let result = license
            .validate(&["test_fingerprint".to_string()], &[])
            .await;
        assert!(matches!(result, Err(Error::LicenseExpired { .. })));
        reset_config();
    }

    #[tokio::test]
    async fn test_validation_with_empty_scope() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(get_mock_body())
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let result = license.validate(&[], &[]).await;
        assert!(result.is_ok());
        reset_config();
    }

    #[tokio::test]
    async fn test_license_with_all_attributes() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/actions/validate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "meta": {
                        "ts": "2021-01-01T00:00:00Z",
                        "valid": true,
                        "detail": "is valid",
                        "code": "VALID",
                        "scope": {
                            "fingerprint": "test_fingerprint"
                        }
                    },
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": 5,
                            "maxMachines": 10,
                            "maxCores": 20,
                            "maxUses": 100,
                            "maxProcesses": 5,
                            "protected": true,
                            "suspended": false,
                            "metadata": {
                                "tier": "premium",
                                "features": ["feature_a", "feature_b"]
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy_123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let result = license
            .validate(&["test_fingerprint".to_string()], &[])
            .await;
        assert!(result.is_ok());

        let validated_license = result.unwrap();
        assert_eq!(validated_license.uses, Some(5));
        assert_eq!(validated_license.max_machines, Some(10));
        assert_eq!(validated_license.max_cores, Some(20));
        assert_eq!(validated_license.max_uses, Some(100));
        assert_eq!(validated_license.max_processes, Some(5));
        assert_eq!(validated_license.protected, Some(true));
        assert_eq!(validated_license.suspended, Some(false));
        assert!(validated_license.metadata.contains_key("tier"));
        assert_eq!(
            validated_license
                .metadata
                .get("tier")
                .unwrap()
                .as_str()
                .unwrap(),
            "premium"
        );

        reset_config();
    }

    #[tokio::test]
    async fn test_machine_activation_errors() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/machines")
            .with_status(422)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "errors": [{
                        "title": "Unprocessable Entity",
                        "detail": "License has reached machine limit",
                        "code": "MACHINE_LIMIT_EXCEEDED"
                    }]
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let result = license.activate("test_fingerprint", &[]).await;
        assert!(result.is_err());
        reset_config();
    }

    #[test]
    fn test_license_relationships() {
        use crate::{
            KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
        };

        // Test that all relationship IDs are properly extracted
        let license_data = KeygenResponseData {
            id: "test-license-id".to_string(),
            r#type: "licenses".to_string(),
            attributes: LicenseAttributes {
                key: "TEST-LICENSE-KEY".to_string(),
                name: Some("Test License".to_string()),
                expiry: None,
                status: Some("active".to_string()),
                uses: Some(5),
                max_machines: Some(10),
                max_cores: Some(20),
                max_uses: Some(100),
                max_processes: Some(5),
                max_users: None,
                protected: Some(true),
                suspended: Some(false),
                permissions: None,
                metadata: HashMap::new(),
            },
            relationships: KeygenRelationships {
                policy: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "policies".to_string(),
                        id: "test-policy-id".to_string(),
                    }),
                    links: None,
                }),
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
                group: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "groups".to_string(),
                        id: "test-group-id".to_string(),
                    }),
                    links: None,
                }),
                owner: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "users".to_string(),
                        id: "test-owner-id".to_string(),
                    }),
                    links: None,
                }),
                users: None,
                machines: None,
                environment: None,
                license: None,
                other: HashMap::new(),
            },
        };

        let license = License::from(license_data);

        assert_eq!(license.policy, Some("test-policy-id".to_string()));
        assert_eq!(license.account_id, Some("test-account-id".to_string()));
        assert_eq!(license.product_id, Some("test-product-id".to_string()));
        assert_eq!(license.group_id, Some("test-group-id".to_string()));
        assert_eq!(license.owner_id, Some("test-owner-id".to_string()));
        assert_eq!(license.id, "test-license-id");
        assert_eq!(license.key, "TEST-LICENSE-KEY");
    }

    #[test]
    fn test_license_without_relationships() {
        use crate::{KeygenRelationships, KeygenResponseData};

        // Test that all relationship IDs are None when no relationships exist
        let license_data = KeygenResponseData {
            id: "test-license-id".to_string(),
            r#type: "licenses".to_string(),
            attributes: LicenseAttributes {
                key: "TEST-LICENSE-KEY".to_string(),
                name: Some("Test License".to_string()),
                expiry: None,
                status: Some("active".to_string()),
                uses: None,
                max_machines: None,
                max_cores: None,
                max_uses: None,
                max_processes: None,
                max_users: None,
                protected: None,
                suspended: None,
                permissions: None,
                metadata: HashMap::new(),
            },
            relationships: KeygenRelationships {
                policy: None,
                account: None,
                product: None,
                group: None,
                owner: None,
                users: None,
                machines: None,
                environment: None,
                license: None,
                other: HashMap::new(),
            },
        };

        let license = License::from(license_data);

        assert_eq!(license.policy, None);
        assert_eq!(license.account_id, None);
        assert_eq!(license.product_id, None);
        assert_eq!(license.group_id, None);
        assert_eq!(license.owner_id, None);
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_basic() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-123",
                        "type": "licenses",
                        "attributes": {
                            "key": "LICENSE-KEY-123",
                            "name": "Test License",
                            "expiry": null,
                            "status": "active",
                            "uses": null,
                            "maxMachines": 5,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {
                                "tier": "premium"
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            },
                            "owner": {
                                "data": {
                                    "type": "users",
                                    "id": "user-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("premium"));

        let request = LicenseCreateRequest::new("policy-123".to_string())
            .with_name("Test License".to_string())
            .with_max_machines(5)
            .with_owner_id("user-123".to_string())
            .with_metadata(metadata);

        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-123");
        assert_eq!(license.key, "LICENSE-KEY-123");
        assert_eq!(license.name, Some("Test License".to_string()));
        assert_eq!(license.max_machines, Some(5));
        assert_eq!(license.owner_id, Some("user-123".to_string()));
        assert!(license.metadata.contains_key("tier"));

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_with_custom_key() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-456",
                        "type": "licenses",
                        "attributes": {
                            "key": "CUSTOM-LICENSE-KEY",
                            "name": "Custom License",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": 10,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-456"
                                }
                            },
                            "group": {
                                "data": {
                                    "type": "groups",
                                    "id": "group-456"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );

        let request = LicenseCreateRequest::new("policy-456".to_string())
            .with_name("Custom License".to_string())
            .with_key("CUSTOM-LICENSE-KEY".to_string())
            .with_expiry(expiry)
            .with_max_machines(10)
            .with_group_id("group-456".to_string());

        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-456");
        assert_eq!(license.key, "CUSTOM-LICENSE-KEY");
        assert_eq!(license.name, Some("Custom License".to_string()));
        assert_eq!(license.max_machines, Some(10));
        assert_eq!(license.group_id, Some("group-456".to_string()));
        assert!(license.owner_id.is_none());

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_minimal() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-789",
                        "type": "licenses",
                        "attributes": {
                            "key": "AUTO-GENERATED-KEY",
                            "name": null,
                            "expiry": null,
                            "status": "active",
                            "uses": null,
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-789"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let request = LicenseCreateRequest::new("policy-789".to_string());
        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-789");
        assert_eq!(license.key, "AUTO-GENERATED-KEY");
        assert!(license.name.is_none());
        assert!(license.max_machines.is_none());
        assert!(license.owner_id.is_none());
        assert!(license.group_id.is_none());

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_error() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(422)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "errors": [
                        {
                            "title": "Unprocessable Entity",
                            "detail": "Policy is required",
                            "code": "MISSING_POLICY"
                        }
                    ]
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let request = LicenseCreateRequest::new("invalid-policy".to_string());
        let result = License::create(request).await;

        assert!(result.is_err());
        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_license_with_all_parameters() {
        let _m = mock("POST", "/v1/licenses")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "license-comprehensive",
                        "type": "licenses",
                        "attributes": {
                            "key": "COMPREHENSIVE-LICENSE-KEY",
                            "name": "Comprehensive License",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": 10,
                            "maxProcesses": 5,
                            "maxUsers": 3,
                            "maxCores": 8,
                            "maxUses": 100,
                            "protected": true,
                            "suspended": false,
                            "permissions": ["activate", "deactivate", "read"],
                            "metadata": {
                                "tier": "enterprise",
                                "features": ["advanced", "premium"]
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-comprehensive"
                                }
                            },
                            "owner": {
                                "data": {
                                    "type": "users",
                                    "id": "user-comprehensive"
                                }
                            },
                            "group": {
                                "data": {
                                    "type": "groups",
                                    "id": "group-comprehensive"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );
        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("enterprise"));
        metadata.insert("features".to_string(), json!(["advanced", "premium"]));

        let request = LicenseCreateRequest::new("policy-comprehensive".to_string())
            .with_name("Comprehensive License".to_string())
            .with_key("COMPREHENSIVE-LICENSE-KEY".to_string())
            .with_expiry(expiry)
            .with_max_machines(10)
            .with_max_processes(5)
            .with_max_users(3)
            .with_max_cores(8)
            .with_max_uses(100)
            .with_protected(true)
            .with_suspended(false)
            .with_permissions(vec![
                "activate".to_string(),
                "deactivate".to_string(),
                "read".to_string(),
            ])
            .with_metadata(metadata)
            .with_owner_id("user-comprehensive".to_string())
            .with_group_id("group-comprehensive".to_string());

        let result = License::create(request).await;

        assert!(result.is_ok());
        let license = result.unwrap();
        assert_eq!(license.id, "license-comprehensive");
        assert_eq!(license.key, "COMPREHENSIVE-LICENSE-KEY");
        assert_eq!(license.name, Some("Comprehensive License".to_string()));
        assert_eq!(license.max_machines, Some(10));
        assert_eq!(license.max_processes, Some(5));
        assert_eq!(license.max_cores, Some(8));
        assert_eq!(license.max_uses, Some(100));
        assert_eq!(license.protected, Some(true));
        assert_eq!(license.suspended, Some(false));
        assert_eq!(license.owner_id, Some("user-comprehensive".to_string()));
        assert_eq!(license.group_id, Some("group-comprehensive".to_string()));
        assert!(license.metadata.contains_key("tier"));
        assert_eq!(
            license.metadata.get("tier").unwrap().as_str().unwrap(),
            "enterprise"
        );

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_license_comprehensive() {
        let license = create_test_license();
        let _m = mock("PATCH", "/v1/licenses/test_license_id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Updated License Name",
                            "expiry": "2025-12-31T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": 20,
                            "maxProcesses": 10,
                            "maxUsers": 5,
                            "maxCores": 16,
                            "maxUses": 200,
                            "protected": true,
                            "suspended": false,
                            "permissions": ["read", "write", "activate"],
                            "metadata": {
                                "tier": "enterprise",
                                "updated": true
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 12, 31)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );
        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("enterprise"));
        metadata.insert("updated".to_string(), json!(true));

        let request = LicenseUpdateRequest::new()
            .with_name("Updated License Name".to_string())
            .with_expiry(expiry)
            .with_max_machines(20)
            .with_max_processes(10)
            .with_max_users(5)
            .with_max_cores(16)
            .with_max_uses(200)
            .with_protected(true)
            .with_suspended(false)
            .with_permissions(vec![
                "read".to_string(),
                "write".to_string(),
                "activate".to_string(),
            ])
            .with_metadata(metadata);

        let result = license.update(request).await;

        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.id, "test_license_id");
        assert_eq!(
            updated_license.name,
            Some("Updated License Name".to_string())
        );
        assert_eq!(updated_license.max_machines, Some(20));
        assert_eq!(updated_license.max_processes, Some(10));
        assert_eq!(updated_license.max_users, Some(5));
        assert_eq!(updated_license.max_cores, Some(16));
        assert_eq!(updated_license.max_uses, Some(200));
        assert_eq!(updated_license.protected, Some(true));
        assert_eq!(updated_license.suspended, Some(false));
        assert!(updated_license.metadata.contains_key("tier"));
        assert_eq!(
            updated_license
                .metadata
                .get("tier")
                .unwrap()
                .as_str()
                .unwrap(),
            "enterprise"
        );

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_license_clear_limits() {
        let license = create_test_license();
        let _m = mock("PATCH", "/v1/licenses/test_license_id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Test License",
                            "expiry": null,
                            "status": "active",
                            "uses": null,
                            "maxMachines": null,
                            "maxProcesses": null,
                            "maxUsers": null,
                            "maxCores": null,
                            "maxUses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {}
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        // Test clearing limits (setting them to null)
        let request = LicenseUpdateRequest::new()
            .clear_max_machines()
            .clear_max_processes()
            .clear_max_users()
            .clear_max_cores()
            .clear_max_uses();

        let result = license.update(request).await;

        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.max_machines, None);
        assert_eq!(updated_license.max_processes, None);
        assert_eq!(updated_license.max_users, None);
        assert_eq!(updated_license.max_cores, None);
        assert_eq!(updated_license.max_uses, None);

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_license_basic() {
        let license = create_test_license();
        let _m = mock("PATCH", "/v1/licenses/test_license_id")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "test_license_id",
                        "type": "licenses",
                        "attributes": {
                            "key": "TEST-LICENSE-KEY",
                            "name": "Updated Name",
                            "expiry": "2025-06-30T23:59:59Z",
                            "status": "active",
                            "uses": null,
                            "maxMachines": null,
                            "maxCores": null,
                            "maxUses": null,
                            "maxProcesses": null,
                            "protected": null,
                            "suspended": false,
                            "metadata": {
                                "updated": true
                            }
                        },
                        "relationships": {
                            "policy": {
                                "data": {
                                    "type": "policies",
                                    "id": "policy-123"
                                }
                            }
                        }
                    }
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let expiry = Utc.from_utc_datetime(
            &chrono::NaiveDate::from_ymd_opt(2025, 6, 30)
                .unwrap()
                .and_hms_opt(23, 59, 59)
                .unwrap(),
        );
        let mut metadata = HashMap::new();
        metadata.insert("updated".to_string(), json!(true));

        let request = LicenseUpdateRequest::new()
            .with_name("Updated Name".to_string())
            .with_expiry(expiry)
            .with_metadata(metadata);

        let result = license.update(request).await;

        assert!(result.is_ok());
        let updated_license = result.unwrap();
        assert_eq!(updated_license.id, "test_license_id");
        assert_eq!(updated_license.name, Some("Updated Name".to_string()));
        assert!(updated_license.metadata.contains_key("updated"));

        reset_config();
    }

    #[test]
    fn test_license_update_request_builder() {
        let mut metadata = HashMap::new();
        metadata.insert("tier".to_string(), json!("premium"));

        let request = LicenseUpdateRequest::new()
            .with_name("Test License".to_string())
            .with_max_machines(10)
            .with_protected(true)
            .with_metadata(metadata.clone());

        assert_eq!(request.name, Some("Test License".to_string()));
        assert_eq!(request.max_machines, Some(Some(10)));
        assert_eq!(request.protected, Some(true));
        assert_eq!(request.metadata, Some(metadata));

        // Test clearing a limit
        let request_with_clear = LicenseUpdateRequest::new()
            .with_max_machines(5)
            .clear_max_machines();

        assert_eq!(request_with_clear.max_machines, Some(None));

        // Test JSON body conversion
        let body = request.to_json_body();
        let data = body.get("data").unwrap();
        let attributes = data.get("attributes").unwrap();
        assert_eq!(
            attributes.get("name").unwrap().as_str().unwrap(),
            "Test License"
        );
        assert_eq!(attributes.get("maxMachines").unwrap().as_i64().unwrap(), 10);
        assert_eq!(
            attributes.get("protected").unwrap().as_bool().unwrap(),
            true
        );
        assert!(attributes.get("metadata").is_some());
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_license_list_pagination_with_page_number() {
        let _m = mock("GET", "/v1/licenses")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[number]".into(), "2".into()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "15".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "license-1",
                            "type": "licenses",
                            "attributes": {
                                "key": "TEST-LICENSE-1",
                                "name": "Test License 1",
                                "expiry": null,
                                "status": "active",
                                "uses": null,
                                "maxMachines": null,
                                "maxCores": null,
                                "maxUses": null,
                                "maxProcesses": null,
                                "protected": null,
                                "suspended": false,
                                "metadata": {}
                            },
                            "relationships": {
                                "policy": {
                                    "data": {
                                        "type": "policies",
                                        "id": "policy-123"
                                    }
                                }
                            }
                        }
                    ]
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let options = LicenseListOptions {
            page_number: Some(2),
            page_size: Some(15),
            ..Default::default()
        };

        let result = License::list(Some(&options)).await;
        assert!(result.is_ok());
        let licenses = result.unwrap();
        assert_eq!(licenses.len(), 1);
        assert_eq!(licenses[0].id, "license-1");

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_license_list_pagination_with_limit_only() {
        let _m = mock("GET", "/v1/licenses")
            .match_query(mockito::Matcher::UrlEncoded("limit".into(), "5".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": []
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let options = LicenseListOptions {
            limit: Some(5),
            ..Default::default()
        };

        let result = License::list(Some(&options)).await;
        assert!(result.is_ok());

        reset_config();
    }

    #[tokio::test]
    async fn test_pagination_options_with_new_parameters() {
        let license = create_test_license();
        let _m = mock("GET", "/v1/licenses/test_license_id/machines")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("page[number]".into(), "3".into()),
                mockito::Matcher::UrlEncoded("page[size]".into(), "25".into()),
                mockito::Matcher::UrlEncoded("limit".into(), "50".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": []
                })
                .to_string(),
            )
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            product: "test_product".to_string(),
            ..Default::default()
        });

        let pagination_options = PaginationOptions {
            limit: Some(50),
            page_number: Some(3),
            page_size: Some(25),
            ..Default::default()
        };

        let result = license.machines(Some(&pagination_options)).await;
        assert!(result.is_ok());
        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_attach_entitlements() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids = vec![
            "entitlement-1".to_string(),
            "entitlement-2".to_string(),
        ];

        let result = license.attach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_detach_entitlements() {
        let license = create_test_license();
        let _m = mock("DELETE", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids = vec![
            "entitlement-1".to_string(),
            "entitlement-2".to_string(),
        ];

        let result = license.detach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_attach_entitlements_empty_list() {
        let license = create_test_license();
        let _m = mock("POST", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids: Vec<String> = vec![];
        let result = license.attach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_detach_entitlements_empty_list() {
        let license = create_test_license();
        let _m = mock("DELETE", "/v1/licenses/test_license_id/entitlements")
            .with_status(204)
            .with_header("content-type", "application/json")
            .create();

        set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let entitlement_ids: Vec<String> = vec![];
        let result = license.detach_entitlements(&entitlement_ids).await;
        assert!(result.is_ok());
        reset_config();
    }

}
