use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    certificate::Certificate, decryptor::Decryptor, errors::Error, license::LicenseAttributes,
    KeygenResponseData,
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
    pub data: KeygenResponseData<LicenseAttributes>,
    pub meta: LicenseFileMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFile {
    pub id: String,
    pub license_id: String,
    pub attributes: LicenseFileAttributes,
}

impl LicenseFile {
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
        let dataset: LicenseFileDataset =
            serde_json::from_slice(&data).map_err(|_| Error::LicenseFileInvalid)?;

        let config = crate::config::get_config();
        if let Some(max_clock_drift) = config.max_clock_drift {
            if Utc::now().signed_duration_since(dataset.meta.issued) > max_clock_drift {
                return Err(Error::SystemClockUnsynced);
            }
        }
        if dataset.meta.ttl != 0 && Utc::now() > dataset.meta.expiry {
            return Err(Error::LicenseFileExpired);
        }

        Ok(dataset)
    }

    fn certificate(&self) -> Result<Certificate, Error> {
        let payload = self.attributes.certificate.trim();
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
