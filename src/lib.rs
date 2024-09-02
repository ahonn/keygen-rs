use client::Client;
use errors::Error;
use license::{License, LicenseResponse, SchemeCode};
use serde::{Deserialize, Serialize};

pub(crate) mod certificate;
pub(crate) mod client;
pub(crate) mod decryptor;
pub(crate) mod entitlement;
pub(crate) mod verifier;

pub mod component;
pub mod config;
pub mod errors;
pub mod license;
pub mod license_file;
pub mod machine;
pub mod machine_file;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenRelationshipData {
  pub r#type: String,
  pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenRelationship {
  data: KeygenRelationshipData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenRelationships {
  pub policy: Option<KeygenRelationship>,
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
/// # Exampled
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
    let client = Client::default();
    let response = client.get("me", None::<&()>).await?;
    let profile: LicenseResponse<()> = serde_json::from_value(response.body)?;
    let license = License::from(profile.data);
    Ok(license.validate_key(fingerprints, entitlements).await?)
}

/// Verifies a signed key based on a given scheme
///
/// Supported schemes are:
/// - Ed25519Sign
///
/// # Exampled
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

pub fn verify(scheme: SchemeCode, signed_key: &str) -> Result<Vec<u8>, Error> {
    let license = License::from_signed_key(scheme, signed_key);
    license.verify()
}
