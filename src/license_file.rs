use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    certificate::Certificate, config::get_config, decryptor::Decryptor, errors::Error, license::{License, LicenseAttributes}, verifier::Verifier, KeygenResponseData
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFileAttributes {
    pub certificate: String,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LicenseFileResponse {
    pub data: KeygenResponseData<LicenseFileAttributes>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseFileMeta {
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseFileDataset {
    pub license: License,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFile {
    pub id: String,
    pub certificate: String,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

impl LicenseFile {
    pub(crate) fn from(data: KeygenResponseData<LicenseFileAttributes>) -> LicenseFile {
        LicenseFile {
            id: data.id,
            certificate: data.attributes.certificate,
            issued: data.attributes.issued,
            expiry: data.attributes.expiry,
            ttl: data.attributes.ttl,
        }
    }

    pub fn verify(&self) -> Result<(), Error> {
        let config = get_config();

        if let Some(public_key) = config.public_key {
            let verifier = Verifier::new(public_key);
            verifier.verify_license_file(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    pub fn decrypt(&self, key: &str) -> Result<LicenseFileDataset, Error> {
        let cert = self.certificate()?;

        match cert.alg.as_str() {
            "aes-256-gcm+rsa-pss-sha256" | "aes-256-gcm+rsa-sha256" => {
                return Err(Error::LicenseFileNotSupported);
            }
            "aes-256-gcm+ed25519" => {}
            _ => return Err(Error::LicenseFileNotEncrypted),
        }

        let decryptor = Decryptor::new(key.to_string());
        let data = decryptor.decrypt_certificate(&cert)?;
        let dataset: Value =
            serde_json::from_slice(&data).map_err(|_| Error::LicenseFileInvalid)?;

        let data: KeygenResponseData<LicenseAttributes> =
            serde_json::from_value(dataset["data"].clone())
                .map_err(|_| Error::LicenseFileInvalid)?;
        let meta: LicenseFileMeta = serde_json::from_value(dataset["meta"].clone())
            .map_err(|_| Error::LicenseFileInvalid)?;
        let license = License::from(data);

        let config = crate::config::get_config();
        if let Some(max_clock_drift) = config.max_clock_drift {
            if Utc::now().signed_duration_since(meta.issued) > max_clock_drift {
                return Err(Error::SystemClockUnsynced);
            }
        }
        if meta.ttl != 0 && Utc::now() > meta.expiry {
            return Err(Error::LicenseFileExpired);
        }

        let dataset = LicenseFileDataset {
            license,
            issued: meta.issued,
            expiry: meta.expiry,
            ttl: meta.ttl,
        };
        Ok(dataset)
    }

    pub(crate) fn certificate(&self) -> Result<Certificate, Error> {
        let payload = self.certificate.trim();
        let payload = payload
            .strip_prefix("-----BEGIN LICENSE FILE-----")
            .and_then(|s| s.strip_suffix("-----END LICENSE FILE-----"))
            .ok_or(Error::LicenseFileInvalid)?
            .trim()
            .replace("\n", "");

        let decoded = general_purpose::STANDARD
            .decode(payload)
            .map_err(|_| Error::LicenseFileInvalid)?;

        let cert: Certificate =
            serde_json::from_slice(&decoded).map_err(|_| Error::LicenseFileInvalid)?;

        Ok(cert)
    }
}
