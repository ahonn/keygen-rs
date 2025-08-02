use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    certificate::{
        validate_certificate_meta, Certificate, CertificateFileAttributes, CertificateFileMeta,
    },
    component::Component,
    config::get_config,
    decryptor::Decryptor,
    entitlement::Entitlement,
    errors::Error,
    license::License,
    license_file::IncludedResources,
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
    #[serde(default)]
    pub included: Option<IncludedResources>,
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
        self.validate_ttl()?;

        let config = get_config()?;

        if let Some(public_key) = config.public_key {
            let verifier = Verifier::new(public_key);
            verifier.verify_machine_file(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    pub fn validate_ttl(&self) -> Result<(), Error> {
        let now = Utc::now();
        if now > self.expiry {
            let dataset = self.decrypt("").unwrap_or_else(|_| {
                use std::collections::HashMap;
                MachineFileDataset {
                    license: License::from(crate::KeygenResponseData {
                        id: "".to_string(),
                        r#type: "licenses".to_string(),
                        attributes: crate::license::LicenseAttributes {
                            name: None,
                            key: "".to_string(),
                            expiry: None,
                            status: Some("".to_string()),
                            uses: Some(0),
                            max_machines: None,
                            max_cores: None,
                            max_uses: None,
                            max_processes: None,
                            max_users: None,
                            protected: None,
                            suspended: None,
                            permissions: None,
                            metadata: HashMap::new(),
                        },
                        relationships: crate::KeygenRelationships::default(),
                    }),
                    machine: Machine::from(crate::KeygenResponseData {
                        id: "".to_string(),
                        r#type: "machines".to_string(),
                        attributes: crate::machine::MachineAttributes {
                            fingerprint: "".to_string(),
                            name: None,
                            platform: None,
                            hostname: None,
                            ip: None,
                            cores: None,
                            metadata: None,
                            require_heartbeat: false,
                            heartbeat_status: "".to_string(),
                            heartbeat_duration: None,
                            created: Utc::now(),
                            updated: Utc::now(),
                        },
                        relationships: crate::KeygenRelationships::default(),
                    }),
                    issued: self.issued,
                    expiry: self.expiry,
                    ttl: self.ttl,
                    included: None,
                }
            });
            Err(Error::MachineFileExpired(dataset))
        } else {
            Ok(())
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

        // Parse included relationships
        let included_array = dataset["included"]
            .as_array()
            .ok_or(Error::MachineFileInvalid(
                "Included data is not an array".into(),
            ))?;

        // Find type = "licenses" element in dataset["included"] array
        let license_data = included_array
            .iter()
            .find(|v| v["type"] == "licenses")
            .ok_or(Error::MachineFileInvalid(
                "No license data found in included data".into(),
            ))?;
        let license = License::from(serde_json::from_value(license_data.clone())?);

        // Parse other included relationships if present
        let included = if included_array.len() > 1 {
            Some(Self::_parse_included(&Value::Array(included_array.clone()))?)
        } else {
            None
        };

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
            included,
        };

        if let Err(err) = validate_certificate_meta(&meta) {
            match err {
                Error::CertificateFileExpired => Err(Error::MachineFileExpired(dataset)),
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

    /// Get entitlements from the machine file without making an API call
    /// Requires the decryption key and the machine file to include entitlements
    pub fn entitlements(&self, key: &str) -> Result<Vec<Entitlement>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_entitlements().unwrap_or(&vec![]).clone())
    }

    /// Get machines from the machine file without making an API call
    /// Requires the decryption key and the machine file to include machines
    pub fn machines(&self, key: &str) -> Result<Vec<Machine>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_machines().unwrap_or(&vec![]).clone())
    }

    /// Get components from the machine file without making an API call
    /// Requires the decryption key and the machine file to include components
    pub fn components(&self, key: &str) -> Result<Vec<Component>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_components().unwrap_or(&vec![]).clone())
    }

    fn _parse_included(included_value: &Value) -> Result<IncludedResources, Error> {
        let mut included = IncludedResources {
            entitlements: Vec::new(),
            machines: Vec::new(),
            components: Vec::new(),
        };

        if let Some(included_array) = included_value.as_array() {
            for item in included_array {
                if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                    match item_type {
                        "entitlements" => {
                            if let Ok(entitlement_data) = serde_json::from_value::<
                                KeygenResponseData<crate::entitlement::EntitlementAttributes>,
                            >(item.clone())
                            {
                                included
                                    .entitlements
                                    .push(Entitlement::from(entitlement_data));
                            }
                        }
                        "machines" => {
                            if let Ok(machine_data) = serde_json::from_value::<
                                KeygenResponseData<crate::machine::MachineAttributes>,
                            >(item.clone())
                            {
                                included.machines.push(Machine::from(machine_data));
                            }
                        }
                        "components" => {
                            // Components might be in a different format, let's try to parse them properly
                            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                if let Some(attrs) = item.get("attributes") {
                                    if let (Some(fingerprint), Some(name)) = (
                                        attrs.get("fingerprint").and_then(|f| f.as_str()),
                                        attrs.get("name").and_then(|n| n.as_str()),
                                    ) {
                                        included.components.push(Component {
                                            id: id.to_string(),
                                            fingerprint: fingerprint.to_string(),
                                            name: name.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        "licenses" => {
                            // Skip licenses as they are handled separately
                        }
                        _ => {
                            // Ignore other types
                        }
                    }
                }
            }
        }

        Ok(included)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machine::MachineCheckoutOpts;
    use serde_json::json;

    #[test]
    fn test_machine_file_included_resources_parsing() {
        // Test parsing of included relationships from JSON API format
        let included_json = json!([
            {
                "type": "licenses",
                "id": "lic1",
                "attributes": {
                    "name": "Test License",
                    "key": "test-key",
                    "expiry": null,
                    "status": "active",
                    "uses": 0,
                    "maxMachines": 5,
                    "maxCores": null,
                    "maxUses": null,
                    "maxProcesses": null,
                    "maxUsers": null,
                    "protected": false,
                    "suspended": false,
                    "permissions": null,
                    "metadata": {}
                },
                "relationships": {
                    "account": {"data": {"type": "accounts", "id": "acc1"}}
                }
            },
            {
                "type": "entitlements",
                "id": "ent1",
                "attributes": {
                    "name": "Feature A",
                    "code": "feature-a",
                    "metadata": {},
                    "created": "2023-01-01T00:00:00Z",
                    "updated": "2023-01-01T00:00:00Z"
                },
                "relationships": {
                    "account": {"data": {"type": "accounts", "id": "acc1"}}
                }
            },
            {
                "type": "components",
                "id": "comp1",
                "attributes": {
                    "fingerprint": "component-fingerprint",
                    "name": "CPU Component"
                }
            }
        ]);

        let result = MachineFile::_parse_included(&included_json);
        assert!(result.is_ok());

        let included = result.unwrap();
        assert_eq!(included.entitlements.len(), 1);
        assert_eq!(included.entitlements[0].code, "feature-a");
        assert_eq!(included.entitlements[0].name, Some("Feature A".to_string()));

        assert_eq!(included.machines.len(), 0);

        assert_eq!(included.components.len(), 1);
        assert_eq!(included.components[0].id, "comp1");
        assert_eq!(included.components[0].fingerprint, "component-fingerprint");
        assert_eq!(included.components[0].name, "CPU Component");
    }

    #[test]
    fn test_machine_checkout_opts_with_ttl() {
        let opts = MachineCheckoutOpts::with_ttl(7200);

        assert_eq!(opts.ttl, Some(7200));
        assert!(opts.include.is_none());
    }

    #[test]
    fn test_machine_checkout_opts_with_include() {
        let include_vec = vec![
            "license.entitlements".to_string(),
            "components".to_string(),
        ];
        let opts = MachineCheckoutOpts::with_include(include_vec);

        assert!(opts.include.is_some());
        let includes = opts.include.unwrap();
        assert!(includes.contains(&"license.entitlements".to_string()));
        assert!(includes.contains(&"components".to_string()));
        assert_eq!(includes.len(), 2);
        assert!(opts.ttl.is_none());
    }

    #[test]
    fn test_machine_checkout_opts_new() {
        let opts = MachineCheckoutOpts::new();

        assert!(opts.ttl.is_none());
        assert!(opts.include.is_none());
    }
}

impl MachineFileDataset {
    /// Get cached entitlements without making an API call
    pub fn offline_entitlements(&self) -> Option<&Vec<Entitlement>> {
        self.included.as_ref().map(|inc| &inc.entitlements)
    }

    /// Get cached machines without making an API call
    pub fn offline_machines(&self) -> Option<&Vec<Machine>> {
        self.included.as_ref().map(|inc| &inc.machines)
    }

    /// Get cached components without making an API call
    pub fn offline_components(&self) -> Option<&Vec<Component>> {
        self.included.as_ref().map(|inc| &inc.components)
    }
}
