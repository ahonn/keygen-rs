use client::Client;
use errors::Error;
use license::{License, LicenseAttributes};
use serde::{Deserialize, Serialize};

pub mod config;
pub mod errors;
pub mod license;
pub mod machine;
pub mod component;
pub(crate) mod artifact;
pub(crate) mod certificate;
pub(crate) mod client;
pub(crate) mod decryptor;
pub(crate) mod entitlement;
pub(crate) mod license_file;
pub(crate) mod log;
pub(crate) mod machine_file;
pub(crate) mod process;
pub(crate) mod release;
pub(crate) mod upgrade;
pub(crate) mod verifier;
pub(crate) mod webhook;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct KeygenResponseData<T> {
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
    let license = License {
        id: profile.data.id,
        scheme: Some(license::SchemeCode::Ed25519Sign),
        attributes: profile.data.attributes,
    };
    Ok(license.validate_key(fingerprints).await?)
}
