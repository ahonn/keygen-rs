use client::Client;
use errors::Error;
use license::{License, LicenseAttributes};
use serde::{Deserialize, Serialize};

pub mod artifact;
pub mod certificate;
pub mod client;
pub mod component;
pub mod config;
pub mod decryptor;
pub mod entitlement;
pub mod errors;
pub mod license;
pub mod license_file;
pub mod log;
pub mod machine;
pub mod machine_file;
pub mod process;
pub mod release;
pub mod upgrade;
pub mod verifier;
pub mod webhook;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeygenResponseData<T> {
    pub id: String,
    pub r#type: String,
    pub attributes: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LicenseProfileResponse {
    pub data: KeygenResponseData<LicenseAttributes>,
}

pub async fn validate(fingerprints: &[String]) -> Result<License, Error> {
    let client = Client::default();
    let response = client.get("me", None::<&()>).await?;
    let profile: LicenseProfileResponse = serde_json::from_value(response.body)?;
    let mut license = License {
        id: profile.data.id,
        name: profile.data.attributes.name.unwrap_or("".to_string()),
        key: profile.data.attributes.key,
        expiry: profile.data.attributes.expiry,
        scheme: Some(license::SchemeCode::Ed25519Sign),
        last_validation: None,
    };
    Ok(license.validate_key(fingerprints).await?)
}
