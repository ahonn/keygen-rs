use std::env;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::certificate::CartificateFileResponse;
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
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
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
    pub protected: Option<bool>,
    pub suspended: Option<bool>,
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

#[derive(Debug, Default)]
pub struct PaginationOptions {
    pub limit: Option<i32>,
    pub page: Option<i32>,
    pub offset: Option<i32>,
}

/// Simple license list options with common filters
#[derive(Debug, Default)]
pub struct LicenseListOptions {
    // Pagination
    pub limit: Option<i32>,
    pub page: Option<i32>,
    pub offset: Option<i32>,

    // Common filters
    pub status: Option<String>,  // "ACTIVE", "EXPIRED", "SUSPENDED", etc.
    pub product: Option<String>, // Product ID
    pub policy: Option<String>,  // Policy ID
    pub owner: Option<String>,   // Owner ID or email
    pub user: Option<String>,    // User ID or email
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
            protected: data.attributes.protected,
            suspended: data.attributes.suspended,
            policy: data.relationships.policy.as_ref().map(|p| p.data.id.clone()),
            metadata: data.attributes.metadata,
            account_id: data.relationships.account.as_ref().map(|a| a.data.id.clone()),
            product_id: data.relationships.product.as_ref().map(|p| p.data.id.clone()),
            group_id: data.relationships.group.as_ref().map(|g| g.data.id.clone()),
            owner_id: data.relationships.owner.as_ref().map(|o| o.data.id.clone()),
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
            protected: None,
            suspended: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
        }
    }

    fn build_scope(fingerprints: &[String], entitlements: &[String]) -> Value {
        let config = get_config();
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

        scope
    }

    pub async fn validate(
        self,
        fingerprints: &[String],
        entitlements: &[String],
    ) -> Result<License, Error> {
        let client = Client::default();
        let scope = Self::build_scope(fingerprints, entitlements);
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
        let client = Client::default();
        let scope = Self::build_scope(fingerprints, entitlements);
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
        let config = get_config();
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
        let config = get_config();
        let client = Client::default();
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
            params["data"]["relationships"]["components"] = json!(components
                .iter()
                .map(|comp| Component::create_object(comp))
                .collect::<Vec<serde_json::Value>>());
        }

        let response = client.post("machines", Some(&params), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        let machine = Machine::from(machine_response.data);
        Ok(machine)
    }

    pub async fn deactivate(&self, id: &str) -> Result<(), Error> {
        let client = Client::default();
        let _response = client
            .delete::<(), serde_json::Value>(&format!("machines/{}", id), None::<&()>)
            .await?;
        Ok(())
    }

    pub async fn machine(&self, id: &str) -> Result<Machine, Error> {
        let client = Client::default();
        let response = client.get(&format!("machines/{}", id), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        let machine = Machine::from(machine_response.data);
        Ok(machine)
    }

    pub async fn machines(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<Machine>, Error> {
        let client = Client::default();
        let mut query = json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            } else {
                query["limit"] = json!(100);
            }

            if let Some(page) = opts.page {
                query["page"] = json!(page);
            }

            if let Some(offset) = opts.offset {
                query["offset"] = json!(offset);
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
        let client = Client::default();
        let mut query = json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            } else {
                query["limit"] = json!(100);
            }

            if let Some(page) = opts.page {
                query["page"] = json!(page);
            }

            if let Some(offset) = opts.offset {
                query["offset"] = json!(offset);
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
        let client = Client::default();
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
        let license_file_response: CartificateFileResponse = serde_json::from_value(response.body)?;
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

    /// Create a new license
    #[cfg(feature = "token")]
    pub async fn create(
        policy_id: &str,
        user_id: Option<&str>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<License, Error> {
        let client = Client::default();

        let mut relationships = serde_json::Map::new();
        relationships.insert(
            "policy".to_string(),
            json!({
                "data": {
                    "type": "policies",
                    "id": policy_id
                }
            }),
        );

        if let Some(uid) = user_id {
            relationships.insert(
                "user".to_string(),
                json!({
                    "data": {
                        "type": "users",
                        "id": uid
                    }
                }),
            );
        }

        let body = json!({
            "data": {
                "type": "licenses",
                "attributes": {
                    "metadata": metadata.unwrap_or_default()
                },
                "relationships": relationships
            }
        });

        let response = client.post("licenses", Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// List all licenses with optional filtering
    #[cfg(feature = "token")]
    pub async fn list(options: Option<&LicenseListOptions>) -> Result<Vec<License>, Error> {
        let client = Client::default();
        let mut query = json!({});

        if let Some(opts) = options {
            // Pagination
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            }
            if let Some(page) = opts.page {
                query["page"] = json!(page);
            }
            if let Some(offset) = opts.offset {
                query["offset"] = json!(offset);
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
        let client = Client::default();
        let endpoint = format!("licenses/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Update a license
    #[cfg(feature = "token")]
    pub async fn update(
        &self,
        name: Option<String>,
        expiry: Option<DateTime<Utc>>,
        metadata: Option<HashMap<String, Value>>,
    ) -> Result<License, Error> {
        let client = Client::default();
        let endpoint = format!("licenses/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(n) = name {
            attributes.insert("name".to_string(), json!(n));
        }
        if let Some(exp) = expiry {
            attributes.insert("expiry".to_string(), json!(exp));
        }
        if let Some(meta) = metadata {
            attributes.insert("metadata".to_string(), json!(meta));
        }

        let body = json!({
            "data": {
                "type": "licenses",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Delete a license
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default();
        let endpoint = format!("licenses/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Suspend a license
    #[cfg(feature = "token")]
    pub async fn suspend(&self) -> Result<License, Error> {
        let client = Client::default();
        let endpoint = format!("licenses/{}/actions/suspend", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Reinstate a suspended license
    #[cfg(feature = "token")]
    pub async fn reinstate(&self) -> Result<License, Error> {
        let client = Client::default();
        let endpoint = format!("licenses/{}/actions/reinstate", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Renew a license
    #[cfg(feature = "token")]
    pub async fn renew(&self) -> Result<License, Error> {
        let client = Client::default();
        let endpoint = format!("licenses/{}/actions/renew", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }

    /// Revoke a license
    #[cfg(feature = "token")]
    pub async fn revoke(&self) -> Result<License, Error> {
        let client = Client::default();
        let endpoint = format!("licenses/{}/actions/revoke", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let license_response: LicenseResponse<()> = serde_json::from_value(response.body)?;
        Ok(License::from(license_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{reset_config, set_config, KeygenConfig};
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
            protected: None,
            suspended: None,
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
                mockito::Matcher::UrlEncoded("page".into(), "2".into()),
                mockito::Matcher::UrlEncoded("offset".into(), "10".into()),
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
            page: Some(2),
            offset: Some(10),
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
        use crate::{KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData};
        
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
                protected: Some(true),
                suspended: Some(false),
                metadata: HashMap::new(),
            },
            relationships: KeygenRelationships {
                policy: Some(KeygenRelationship {
                    data: KeygenRelationshipData {
                        r#type: "policies".to_string(),
                        id: "test-policy-id".to_string(),
                    },
                }),
                account: Some(KeygenRelationship {
                    data: KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    },
                }),
                product: Some(KeygenRelationship {
                    data: KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    },
                }),
                group: Some(KeygenRelationship {
                    data: KeygenRelationshipData {
                        r#type: "groups".to_string(),
                        id: "test-group-id".to_string(),
                    },
                }),
                owner: Some(KeygenRelationship {
                    data: KeygenRelationshipData {
                        r#type: "users".to_string(),
                        id: "test-owner-id".to_string(),
                    },
                }),
                users: None,
                machines: None,
                environment: None,
                license: None,
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
                protected: None,
                suspended: None,
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
            },
        };

        let license = License::from(license_data);
        
        assert_eq!(license.policy, None);
        assert_eq!(license.account_id, None);
        assert_eq!(license.product_id, None);
        assert_eq!(license.group_id, None);
        assert_eq!(license.owner_id, None);
    }
}
