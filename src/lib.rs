use client::Client;
use errors::Error;
use license::{License, LicenseResponse, SchemeCode};
use serde::{Deserialize, Serialize};

pub(crate) mod artifact;
pub(crate) mod certificate;
pub(crate) mod client;
pub mod component;
pub mod config;
pub(crate) mod decryptor;
pub(crate) mod entitlement;
pub mod errors;
pub mod license;
pub(crate) mod license_file;
pub(crate) mod log;
pub mod machine;
pub(crate) mod machine_file;
pub(crate) mod process;
pub(crate) mod release;
pub(crate) mod upgrade;
pub(crate) mod verifier;
pub(crate) mod webhook;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeygenResponseData<T> {
    pub id: String,
    pub r#type: String,
    pub attributes: T,
}

pub async fn validate(fingerprints: &[String]) -> Result<License, Error> {
    let client = Client::default();
    let response = client.get("me", None::<&()>).await?;
    let profile: LicenseResponse<()> = serde_json::from_value(response.body)?;
    let license = License::from(profile.data);
    Ok(license.validate_key(fingerprints).await?)
}

pub fn verify(scheme: SchemeCode, signed_key: &str) -> Result<Vec<u8>, Error> {
    let license = License::from_signed_key(scheme, signed_key);
    license.verify()
}
