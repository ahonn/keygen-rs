use std::env;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::client::Client;
use crate::component::Component;
use crate::entitlement::Entitlement;
use crate::errors::Error;
use crate::{get_config, reset_config};
use crate::license_file::LicenseFile;
use crate::machine::Machine;
use crate::verifier::Verifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemeCode {
    #[serde(rename = "ED25519_SIGN")]
    Ed25519Sign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResponse {
    pub meta: ValidationMeta,
    pub data: LicenseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMeta {
    pub ts: DateTime<Utc>,
    pub valid: bool,
    pub detail: String,
    pub code: String,
    pub scope: ValidationScope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationScope {
    pub fingerprint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseData {
    pub id: String,
    pub r#type: String,
    pub attributes: LicenseAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseAttributes {
    pub name: Option<String>,
    pub key: String,
    pub expiry: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub key: String,
    pub expiry: Option<DateTime<Utc>>,
    pub scheme: Option<SchemeCode>,
    pub require_heartbeat: bool,
    pub last_validated: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip)]
    pub last_validation: Option<ValidationResponse>,
}

pub struct CheckoutOptions {
    pub ttl: Option<chrono::Duration>,
    pub include: Option<Vec<String>>,
}

impl License {
    pub async fn validate(&mut self, fingerprints: &[String]) -> Result<ValidationResponse, Error> {
        let client = Client::default();
        let scope = License::build_scope(fingerprints);
        let params = json!({
            "meta": {
                "nonce": chrono::Utc::now().timestamp(),
                "scope": scope
            }
        });

        let response = client
            .post(&format!("licenses/{}/actions/validate", self.id), &params)
            .await?;
        let validation_response: ValidationResponse = serde_json::from_value(response.body)?;
        self.last_validation = Some(validation_response.clone());

        if !validation_response.meta.valid {
            return Err(self.handle_validation_code(&validation_response.meta.code));
        };
        Ok(validation_response)
    }

    pub async fn validate_key(
        &mut self,
        fingerprints: &[String],
    ) -> Result<ValidationResponse, Error> {
        let client = Client::default();
        let scope = License::build_scope(fingerprints);
        let params = json!({
            "meta": {
                "key": self.key.clone(),
                "scope": scope
            }
        });

        let response = client
            .post(&"licenses/actions/validate-key", &params)
            .await?;
        let validation_response: ValidationResponse = serde_json::from_value(response.body)?;
        self.last_validation = Some(validation_response.clone());

        if !validation_response.meta.valid {
            return Err(self.handle_validation_code(&validation_response.meta.code));
        };
        Ok(validation_response)
    }

    fn build_scope(fingerprints: &[String]) -> serde_json::Value {
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
        if let Some(env) = config.environment.as_ref() {
            scope["environment"] = json!(env);
        }
        scope
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
        let client = Client::default();
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().into_owned())
            .unwrap_or_else(|_| String::from("unknown"));

        let params = json!({
          "data": {
            "type": "machines",
            "attributes": {
              "fingerprint": fingerprint,
              "cores": num_cpus::get(),
              "hostname": hostname,
              "platform": format!("{}/{}", env::consts::OS, env::consts::ARCH),
            },
            "relationships": {
              "license": {
                "data": {
                  "type": "licenses",
                  "id": self.id
                }
              },
              "components": components.iter().map(|comp| Component::create_object(comp)).collect::<Vec<serde_json::Value>>()
            }
          }
        });

        let response = client.post("machines", &params).await?;
        let machine: Machine = serde_json::from_value(response.body)?;

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
        let machine: Machine = serde_json::from_value(response.body)?;
        Ok(machine)
    }

    pub async fn machines(&self) -> Result<Vec<Machine>, Error> {
        let client = Client::default();
        let response = client
            .get(
                &format!("licenses/{}/machines", self.id),
                Some(&json!({"limit": 100})),
            )
            .await?;
        let machines: Vec<Machine> = serde_json::from_value(response.body)?;
        Ok(machines)
    }

    pub async fn entitlements(&self) -> Result<Vec<Entitlement>, Error> {
        let client = Client::default();
        let response = client
            .get(
                &format!("licenses/{}/entitlements", self.id),
                Some(&json!({"limit": 100})),
            )
            .await?;
        let entitlements: Vec<Entitlement> = serde_json::from_value(response.body)?;
        Ok(entitlements)
    }

    pub async fn checkout(&self, options: &CheckoutOptions) -> Result<LicenseFile, Error> {
        let client = Client::default();
        let mut params = json!({
            "encrypt": true,
            "include": "entitlements"
        });

        if let Some(ttl) = options.ttl {
            params["ttl"] = json!(ttl.num_seconds());
        }

        if let Some(ref include) = options.include {
            params["include"] = json!(include.join(","));
        }

        let response = client
            .post(&format!("licenses/{}/actions/check-out", self.id), &params)
            .await?;
        let license_file: LicenseFile = serde_json::from_value(response.body)?;
        Ok(license_file)
    }

    fn handle_validation_code(&self, code: &str) -> Error {
        match code {
            "FINGERPRINT_SCOPE_MISMATCH" | "NO_MACHINES" | "NO_MACHINE" => {
                Error::LicenseNotActivated
            }
            "EXPIRED" => Error::LicenseExpired,
            "SUSPENDED" => Error::LicenseSuspended,
            "TOO_MANY_MACHINES" => Error::LicenseTooManyMachines,
            "TOO_MANY_CORES" => Error::LicenseTooManyCores,
            "TOO_MANY_PROCESSES" => Error::LicenseTooManyProcesses,
            "FINGERPRINT_SCOPE_REQUIRED" | "FINGERPRINT_SCOPE_EMPTY" => {
                Error::ValidationFingerprintMissing
            }
            "COMPONENTS_SCOPE_REQUIRED" | "COMPONENTS_SCOPE_EMPTY" => {
                Error::ValidationComponentsMissing
            }
            "COMPONENTS_SCOPE_MISMATCH" => Error::ComponentNotActivated,
            "HEARTBEAT_NOT_STARTED" => Error::HeartbeatRequired,
            "HEARTBEAT_DEAD" => Error::HeartbeatDead,
            "PRODUCT_SCOPE_REQUIRED" | "PRODUCT_SCOPE_EMPTY" => Error::ValidationProductMissing,
            _ => Error::LicenseKeyInvalid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{set_config, KeygenConfig};
    use chrono::Utc;
    use mockito::{mock, server_url};
    use serde_json::json;

    fn create_test_license() -> License {
        License {
            id: "test_license_id".to_string(),
            name: "Test License".to_string(),
            key: "TEST-LICENSE-KEY".to_string(),
            expiry: None,
            scheme: Some(SchemeCode::Ed25519Sign),
            require_heartbeat: false,
            last_validated: None,
            created: Utc::now(),
            updated: Utc::now(),
            last_validation: None,
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
                    "status": "valid"
                }
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn test_validate() {
        let mut license = create_test_license();
        let _m = mock(
            "POST",
            "/v1/licenses/test_license_id/actions/validate",
        )
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
            .validate(&[
                "test_fingerprint".to_string(),
                "comp1".to_string(),
                "comp2".to_string(),
            ])
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
        reset_config();
    }

    #[tokio::test]
    async fn test_validate_key() {
        let mut license = create_test_license();
        let _m = mock(
            "POST",
            "/v1/licenses/actions/validate-key",
        )
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
            .validate_key(&[
                "test_fingerprint".to_string(),
                "comp1".to_string(),
                "comp2".to_string(),
            ])
            .await;
        println!("{:?}", result);
        assert!(result.is_ok());
        reset_config();
    }

    #[test]
    fn test_verify() {
        let mut license = create_test_license();

        let result = license.verify();
        assert!(matches!(result, Err(Error::PublicKeyMissing)));

        license.scheme = None;
        let result = license.verify();
        assert!(matches!(result, Err(Error::LicenseNotSigned)));
    }
}
