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
    group::Group,
    license::{License, LicenseAttributes},
    verifier::Verifier,
    KeygenResponseData,
};

/// Container for included relationship data from license/machine checkout
/// For License Checkout: entitlements, group
/// For Machine Checkout: license.entitlements, components, group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludedResources {
    #[serde(default)]
    pub entitlements: Vec<Entitlement>,
    #[serde(default)]
    pub components: Vec<Component>, // Only for machine checkout
    #[serde(default)]
    pub groups: Vec<Group>,
}

impl IncludedResources {
    /// Parse included relationships from JSON API format
    pub fn parse_from_json(included_value: &Value) -> Result<Self, Error> {
        let mut included = Self {
            entitlements: Vec::new(),
            components: Vec::new(),
            groups: Vec::new(),
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
                        "groups" => {
                            // Parse group data - simplified structure
                            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                if let Some(attrs) = item.get("attributes") {
                                    let name = attrs
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("Unknown Group")
                                        .to_string();
                                    let metadata = attrs
                                        .get("metadata")
                                        .and_then(|m| m.as_object())
                                        .map(|obj| {
                                            obj.iter()
                                                .map(|(k, v)| (k.clone(), v.clone()))
                                                .collect()
                                        })
                                        .unwrap_or_default();

                                    included.groups.push(Group {
                                        id: id.to_string(),
                                        name,
                                        metadata,
                                    });
                                }
                            }
                        }
                        "licenses" => {
                            // Skip licenses as they are handled separately in machine files
                        }
                        _ => {
                            // Ignore other types (users, owner, product, policy, environment need special permissions)
                        }
                    }
                }
            }
        }

        Ok(included)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseFileDataset {
    pub license: License,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
    #[serde(default)]
    pub included: Option<IncludedResources>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFile {
    pub id: String,
    pub certificate: String,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

impl From<CertificateFileAttributes> for LicenseFile {
    fn from(val: CertificateFileAttributes) -> Self {
        LicenseFile {
            id: "".into(),
            certificate: val.certificate,
            issued: val.issued,
            expiry: val.expiry,
            ttl: val.ttl,
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
                Error::CertificateFileExpired => Err(Error::LicenseFileExpired(Box::new(dataset))),
                _ => Err(err),
            }
        } else {
            Ok(LicenseFile {
                id: dataset.license.id.clone(),
                certificate: content.to_string(),
                issued: dataset.issued,
                expiry: dataset.expiry,
                ttl: dataset.ttl,
            })
        }
    }

    pub fn verify(&self) -> Result<(), Error> {
        let config = get_config()?;

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

    /// Get entitlements from the license file without making an API call
    /// Requires the decryption key and the license file to include entitlements
    pub fn entitlements(&self, key: &str) -> Result<Vec<Entitlement>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_entitlements().unwrap_or(&vec![]).clone())
    }

    /// Get components from the license file without making an API call
    /// Requires the decryption key and the license file to include components
    pub fn components(&self, key: &str) -> Result<Vec<Component>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_components().unwrap_or(&vec![]).clone())
    }

    /// Get groups from the license file without making an API call
    /// Requires the decryption key and the license file to include groups
    pub fn groups(&self, key: &str) -> Result<Vec<Group>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_groups().unwrap_or(&vec![]).clone())
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

        let data: KeygenResponseData<LicenseAttributes> =
            serde_json::from_value(dataset["data"].clone())
                .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;
        let license = License::from(data);

        // Parse included relationships if present
        let included = if let Some(included_value) = dataset.get("included") {
            if included_value.is_array() && !included_value.as_array().unwrap().is_empty() {
                Some(IncludedResources::parse_from_json(included_value)?)
            } else {
                None
            }
        } else {
            None
        };

        let dataset = LicenseFileDataset {
            license,
            issued: meta.issued,
            expiry: meta.expiry,
            ttl: meta.ttl,
            included,
        };

        if let Err(err) = validate_certificate_meta(&meta) {
            match err {
                Error::CertificateFileExpired => Err(Error::LicenseFileExpired(Box::new(dataset))),
                _ => Err(err),
            }
        } else {
            Ok(dataset)
        }
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

impl LicenseFileDataset {
    /// Get cached entitlements without making an API call
    pub fn offline_entitlements(&self) -> Option<&Vec<Entitlement>> {
        self.included.as_ref().map(|inc| &inc.entitlements)
    }

    /// Get cached components without making an API call
    pub fn offline_components(&self) -> Option<&Vec<Component>> {
        self.included.as_ref().map(|inc| &inc.components)
    }

    /// Get cached groups without making an API call
    pub fn offline_groups(&self) -> Option<&Vec<Group>> {
        self.included.as_ref().map(|inc| &inc.groups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::LicenseCheckoutOpts;
    use serde_json::json;

    #[test]
    fn test_included_resources_parsing() {
        // Test parsing of included relationships from JSON API format
        let included_json = json!([
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

        let result = IncludedResources::parse_from_json(&included_json);
        assert!(result.is_ok());

        let included = result.unwrap();
        assert_eq!(included.entitlements.len(), 1);
        assert_eq!(included.entitlements[0].code, "feature-a");
        assert_eq!(included.entitlements[0].name, Some("Feature A".to_string()));

        assert_eq!(included.components.len(), 1);
        assert_eq!(included.components[0].id, "comp1");
        assert_eq!(included.components[0].fingerprint, "component-fingerprint");
        assert_eq!(included.components[0].name, "CPU Component");
    }

    #[test]
    fn test_license_file_dataset_offline_methods() {
        let included = IncludedResources {
            entitlements: vec![Entitlement {
                id: "ent1".to_string(),
                name: Some("Test Entitlement".to_string()),
                code: "test-code".to_string(),
                metadata: None,
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
                account_id: Some("acc1".to_string()),
            }],
            components: vec![Component {
                id: "comp1".to_string(),
                fingerprint: "test-fingerprint".to_string(),
                name: "Test Component".to_string(),
            }],
            groups: vec![],
        };

        let dataset = LicenseFileDataset {
            license: License {
                id: "lic1".to_string(),
                scheme: None,
                key: "test-key".to_string(),
                name: Some("Test License".to_string()),
                expiry: None,
                status: Some("active".to_string()),
                uses: Some(0),
                max_machines: Some(5),
                max_cores: None,
                max_uses: None,
                max_processes: None,
                max_users: None,
                protected: Some(false),
                suspended: Some(false),
                permissions: None,
                policy: Some("policy1".to_string()),
                metadata: std::collections::HashMap::new(),
                account_id: Some("acc1".to_string()),
                product_id: Some("prod1".to_string()),
                group_id: None,
                owner_id: None,
            },
            issued: chrono::Utc::now(),
            expiry: chrono::Utc::now(),
            ttl: 3600,
            included: Some(included),
        };

        // Test offline access methods
        assert_eq!(dataset.offline_entitlements().unwrap().len(), 1);
        assert_eq!(dataset.offline_entitlements().unwrap()[0].code, "test-code");

        assert_eq!(dataset.offline_components().unwrap().len(), 1);
        assert_eq!(
            dataset.offline_components().unwrap()[0].name,
            "Test Component"
        );
    }

    #[test]
    fn test_license_checkout_opts_with_ttl() {
        let opts = LicenseCheckoutOpts::with_ttl(7200);

        assert_eq!(opts.ttl, Some(7200));
        assert!(opts.include.is_none());
    }

    #[test]
    fn test_license_checkout_opts_with_include() {
        let include_vec = vec![
            "entitlements".to_string(),
            "machines".to_string(),
            "components".to_string(),
        ];
        let opts = LicenseCheckoutOpts::with_include(include_vec);

        assert!(opts.include.is_some());
        let includes = opts.include.unwrap();
        assert!(includes.contains(&"entitlements".to_string()));
        assert!(includes.contains(&"machines".to_string()));
        assert!(includes.contains(&"components".to_string()));
        assert_eq!(includes.len(), 3);
        assert!(opts.ttl.is_none());
    }

    #[test]
    fn test_license_checkout_opts_new() {
        let opts = LicenseCheckoutOpts::new();

        assert!(opts.ttl.is_none());
        assert!(opts.include.is_none());
    }
}
