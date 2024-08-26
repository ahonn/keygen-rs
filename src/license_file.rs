use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    certificate::{
        validate_certificate_meta, Certificate, CertificateFileAttributes, CertificateFileMeta,
    },
    config::get_config,
    decryptor::Decryptor,
    errors::Error,
    license::{License, LicenseAttributes},
    verifier::Verifier,
    KeygenResponseData,
};

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

impl Into<LicenseFile> for CertificateFileAttributes {
    fn into(self) -> LicenseFile {
        LicenseFile {
            id: "".into(),
            certificate: self.certificate,
            issued: self.issued,
            expiry: self.expiry,
            ttl: self.ttl,
        }
    }
}

impl LicenseFile {
    pub(crate) fn from(data: KeygenResponseData<CertificateFileAttributes>) -> LicenseFile {
        LicenseFile {
            id: data.id,
            ..data.attributes.into()
        }
    }

    pub fn from_cert(key: &str, content: &str) -> Result<LicenseFile, Error> {
        let dataset = Self::_decrypt(key, content)?;
        let meta = CertificateFileMeta {
            issued: dataset.issued,
            expiry: dataset.expiry,
            ttl: dataset.ttl,
        };
        if let Err(err) = validate_certificate_meta(&meta) {
            match err {
                Error::CerificateFileExpired => Error::LicenseFileExpired,
                _ => err,
            };
        };

        Ok(LicenseFile {
            id: dataset.license.id.clone(),
            certificate: content.to_string(),
            issued: dataset.issued,
            expiry: dataset.expiry,
            ttl: dataset.ttl,
        })
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
        Self::_decrypt(key, &self.certificate)
    }

    pub fn certificate(&self) -> Result<Certificate, Error> {
        Self::_certificate(self.certificate.clone())
    }

    fn _decrypt(key: &str, content: &str) -> Result<LicenseFileDataset, Error> {
        let cert = Self::_certificate(content.to_string())?;
        match cert.alg.as_str() {
            "aes-256-gcm+rsa-pss-sha256" | "aes-256-gcm+rsa-sha256" => {
                return Err(Error::LicenseFileNotSupported(cert.alg.clone()));
            }
            "aes-256-gcm+ed25519" => {}
            _ => return Err(Error::LicenseFileNotEncrypted),
        }

        let decryptor = Decryptor::new(key.to_string());
        let data = decryptor.decrypt_certificate(&cert)?;
        let dataset: Value =
            serde_json::from_slice(&data).map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;

        let meta: CertificateFileMeta = serde_json::from_value(dataset["meta"].clone())
            .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;
        if let Err(err) = validate_certificate_meta(&meta) {
            match err {
                Error::CerificateFileExpired => Error::LicenseFileExpired,
                _ => err,
            };
        };

        let data: KeygenResponseData<LicenseAttributes> =
            serde_json::from_value(dataset["data"].clone())
                .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;
        let license = License::from(data);

        let dataset = LicenseFileDataset {
            license,
            issued: meta.issued,
            expiry: meta.expiry,
            ttl: meta.ttl,
        };
        Ok(dataset)
    }

    fn _certificate(certificate: String) -> Result<Certificate, Error> {
        let payload = certificate.trim();
        let payload = payload
            .strip_prefix("-----BEGIN LICENSE FILE-----")
            .and_then(|s| s.strip_suffix("-----END LICENSE FILE-----"))
            .ok_or(Error::LicenseFileInvalid(
                "Invalid license file format".into(),
            ))?
            .trim()
            .replace("\n", "");

        let decoded = general_purpose::STANDARD
            .decode(payload)
            .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;

        let cert: Certificate = serde_json::from_slice(&decoded)
            .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;

        Ok(cert)
    }
}
