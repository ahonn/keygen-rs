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
    license::License,
    machine::{Machine, MachineAttributes},
    verifier::Verifier,
    KeygenResponseData,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineFileDataset {
    pub license: License,
    pub machine: Machine,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineFile {
    pub id: String,
    pub certificate: String,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

impl Into<MachineFile> for CertificateFileAttributes {
    fn into(self) -> MachineFile {
        MachineFile {
            id: "".into(),
            certificate: self.certificate,
            issued: self.issued,
            expiry: self.expiry,
            ttl: self.ttl,
        }
    }
}

impl MachineFile {
    pub(crate) fn from(data: KeygenResponseData<CertificateFileAttributes>) -> MachineFile {
        MachineFile {
            id: data.id,
            ..data.attributes.into()
        }
    }

    pub fn from_cert(key: &str, content: &str) -> Result<MachineFile, Error> {
        let dataset = Self::_decrypt(key, content)?;
        Ok(MachineFile {
            id: dataset.machine.id.clone(),
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
            verifier.verify_machine_file(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    pub fn decrypt(&self, key: &str) -> Result<MachineFileDataset, Error> {
        Self::_decrypt(key, &self.certificate)
    }

    pub fn certificate(&self) -> Result<Certificate, Error> {
        Self::_certificate(self.certificate.clone())
    }

    fn _decrypt(key: &str, content: &str) -> Result<MachineFileDataset, Error> {
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
            serde_json::from_slice(&data).map_err(|e| Error::MachineFileInvalid(e.to_string()))?;

        let meta: CertificateFileMeta = serde_json::from_value(dataset["meta"].clone())
            .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;

        // Find type = "licenses" element in dataset["included"] array
        let license_data = dataset["included"]
            .as_array()
            .ok_or(Error::MachineFileInvalid(
                "Included data is not an array".into(),
            ))?
            .iter()
            .find(|v| v["type"] == "licenses")
            .ok_or(Error::MachineFileInvalid(
                "No license data found in included data".into(),
            ))?;
        let license = License::from(serde_json::from_value(license_data.clone())?);

        let machine_data: KeygenResponseData<MachineAttributes> =
            serde_json::from_value(dataset["data"].clone())
                .map_err(|e| Error::MachineFileInvalid(e.to_string()))?;
        let machine = Machine::from(machine_data);

        let dataset = MachineFileDataset {
            license,
            machine,
            issued: meta.issued,
            expiry: meta.expiry,
            ttl: meta.ttl,
        };

        if let Err(err) = validate_certificate_meta(&meta) {
            match err {
                Error::CerificateFileExpired => Err(Error::MachineFileExpired(dataset)),
                _ => Err(err),
            }
        } else {
            Ok(dataset)
        }
    }

    fn _certificate(certificate: String) -> Result<Certificate, Error> {
        let payload = certificate.trim();
        let payload = payload
            .strip_prefix("-----BEGIN MACHINE FILE-----")
            .and_then(|s| s.strip_suffix("-----END MACHINE FILE-----"))
            .ok_or(Error::MachineFileInvalid(
                "Invalid machine file format".into(),
            ))?
            .trim()
            .replace("\n", "");

        let decoded = general_purpose::STANDARD
            .decode(payload)
            .map_err(|e| Error::MachineFileInvalid(e.to_string()))?;

        let cert: Certificate = serde_json::from_slice(&decoded)
            .map_err(|e| Error::MachineFileInvalid(e.to_string()))?;

        Ok(cert)
    }
}
