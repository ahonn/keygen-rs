use client::{Client, ClientOptions};
use config::get_config;
use errors::Error;
use license::{License, SchemeCode};
use serde::{Deserialize, Serialize};

use crate::config::KeygenConfig;

pub(crate) mod certificate;
pub(crate) mod client;
pub(crate) mod decryptor;
pub(crate) mod verifier;

pub mod component;
pub mod config;
pub mod entitlement;
pub mod errors;
pub mod group;
pub mod license;
pub mod license_file;
pub mod machine;
pub mod machine_file;
pub mod service;

// Management features only available with "token" feature flag
#[cfg(feature = "token")]
pub mod environment;
#[cfg(feature = "token")]
pub mod policy;
#[cfg(feature = "token")]
pub mod product;
#[cfg(feature = "token")]
pub mod release;
#[cfg(feature = "token")]
pub mod token;
#[cfg(feature = "token")]
pub mod user;
#[cfg(feature = "token")]
pub mod webhook;

// Distribution modules (only available with "token" feature flag)
#[cfg(feature = "token")]
pub mod arch;
#[cfg(feature = "token")]
pub mod artifact;
#[cfg(feature = "token")]
pub mod channel;
#[cfg(feature = "token")]
pub mod package;
#[cfg(feature = "token")]
pub mod platform;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenRelationshipData {
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenRelationship {
    #[serde(default)]
    pub data: Option<KeygenRelationshipData>,
    #[serde(default)]
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct KeygenRelationships {
    #[serde(default)]
    pub policy: Option<KeygenRelationship>,
    #[serde(default)]
    pub account: Option<KeygenRelationship>,
    #[serde(default)]
    pub product: Option<KeygenRelationship>,
    #[serde(default)]
    pub group: Option<KeygenRelationship>,
    #[serde(default)]
    pub owner: Option<KeygenRelationship>,
    #[serde(default)]
    pub users: Option<KeygenRelationship>,
    #[serde(default)]
    pub machines: Option<KeygenRelationship>,
    #[serde(default)]
    pub environment: Option<KeygenRelationship>,
    #[serde(default)]
    pub license: Option<KeygenRelationship>,
    #[serde(default)]
    pub release: Option<KeygenRelationship>,
    // Use flatten to capture any other relationship fields we don't explicitly handle
    #[serde(flatten)]
    pub other: std::collections::HashMap<String, serde_json::Value>,
}

impl KeygenRelationships {
    /// Extracts the ID from a relationship field
    pub(crate) fn extract_id(rel: &Option<KeygenRelationship>) -> Option<String> {
        rel.as_ref()
            .and_then(|r| r.data.as_ref().map(|d| d.id.clone()))
    }

    pub(crate) fn policy_id(&self) -> Option<String> {
        Self::extract_id(&self.policy)
    }

    pub(crate) fn account_id(&self) -> Option<String> {
        Self::extract_id(&self.account)
    }

    pub(crate) fn product_id(&self) -> Option<String> {
        Self::extract_id(&self.product)
    }

    pub(crate) fn group_id(&self) -> Option<String> {
        Self::extract_id(&self.group)
    }

    pub(crate) fn owner_id(&self) -> Option<String> {
        Self::extract_id(&self.owner)
    }

    pub(crate) fn environment_id(&self) -> Option<String> {
        Self::extract_id(&self.environment)
    }

    pub(crate) fn license_id(&self) -> Option<String> {
        Self::extract_id(&self.license)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenResponseData<T> {
    pub id: String,
    pub r#type: String,
    pub attributes: T,
    pub relationships: KeygenRelationships,
}

/// Validates a license key
///
/// # Example
/// ```
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     dotenv().ok();
///     config::set_config(KeygenConfig {
///         api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
///         account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
///         product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
///         license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
///         public_key: Some(env::var("KEYGEN_PUBLIC_KEY").expect("KEYGEN_PUBLIC_KEY must be set")),
///         ..KeygenConfig::default()
///     });
///
///     let fingerprint = machine_uid::get().unwrap_or("".into());
///     let license = keygen_rs::validate(&[fingerprint]).await?;
///     println!("License validated successfully: {:?}", license);
///     Ok(())
/// }
/// ```
pub async fn validate(fingerprints: &[String], entitlements: &[String]) -> Result<License, Error> {
    let config = get_config()?;
    validate_with_config(config, fingerprints, entitlements).await
}

pub async fn validate_with_config(
    config: KeygenConfig,
    fingerprints: &[String],
    entitlements: &[String],
) -> Result<License, Error> {
    let client = Client::new(ClientOptions::from(config.clone()))?;
    let response = client.get("me", None::<&()>).await?;
    let profile: license::LicenseResponse<()> = serde_json::from_value(response.body)?;
    License::from(profile.data)
        .with_config(config)
        .validate_key(fingerprints, entitlements)
        .await
}

/// Verifies a signed key based on a given scheme
///
/// Supported schemes are:
/// - Ed25519Sign
///
/// # Example
/// ```
/// #[tokio::main]
/// async fn main() {
///     dotenv().ok();
///     let (public_key, signed_key) =
///         generate_signed_license_key("4F5D3B-0FB8B2-6871BC-5D3EB3-4885B7-V3".to_string());
///     config::set_config(KeygenConfig {
///         api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
///         account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
///         product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
///         license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
///         public_key: Some(public_key.clone()),
///         ..KeygenConfig::default()
///     });
///
///     println!("Signed key: {:?}", signed_key);
///     if let Ok(data) = keygen_rs::verify(SchemeCode::Ed25519Sign, &signed_key) {
///       println!("License verified: {:?}", String::from_utf8_lossy(&data));
///     } else {
///       println!("License verification failed");
///     }
/// }
#[must_use = "verification result should be checked"]
pub fn verify(scheme: SchemeCode, signed_key: &str) -> Result<Vec<u8>, Error> {
    let config = get_config()?;
    verify_with_config(&config, scheme, signed_key)
}

#[must_use = "verification result should be checked"]
pub fn verify_with_config(
    config: &config::KeygenConfig,
    scheme: SchemeCode,
    signed_key: &str,
) -> Result<Vec<u8>, Error> {
    License::from_signed_key(scheme, signed_key)
        .with_config(config.clone())
        .verify()
}
