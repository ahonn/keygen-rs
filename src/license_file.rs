use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    certificate::{
        validate_certificate_meta, Certificate, CertificateFileAttributes, CertificateFileMeta,
    },
    component::Component,
    decryptor::Decryptor,
    entitlement::Entitlement,
    errors::Error,
    group::Group,
    license::{License, LicenseAttributes, LicenseFileAlgorithm},
    verifier::Verifier,
    KeygenResponseData,
};

/// Included JSON:API resources from license and machine checkouts.
///
/// `resources` preserves every resource losslessly. The typed collections are
/// compatibility views for resource types supported by earlier SDK versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct IncludedResources {
    #[serde(default)]
    pub resources: Vec<IncludedResource>,
    #[serde(default)]
    pub entitlements: Vec<Entitlement>,
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(default)]
    pub groups: Vec<Group>,
}

/// A lossless JSON:API resource included in an offline license dataset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct IncludedResource {
    pub id: String,
    #[serde(rename = "type")]
    pub resource_type: String,
    #[serde(default)]
    #[cfg_attr(
        feature = "specta",
        specta(type = crate::specta_support::KeygenJsonValue)
    )]
    pub attributes: Value,
    #[serde(default)]
    #[cfg_attr(
        feature = "specta",
        specta(type = crate::specta_support::KeygenJsonValue)
    )]
    pub relationships: Value,
    #[serde(default)]
    #[cfg_attr(
        feature = "specta",
        specta(type = crate::specta_support::KeygenJsonValue)
    )]
    pub links: Value,
}

impl IncludedResources {
    pub fn resources_by_type(&self, resource_type: &str) -> Vec<&IncludedResource> {
        self.resources
            .iter()
            .filter(|resource| resource.resource_type == resource_type)
            .collect()
    }

    /// Parse included resources from a JSON:API `included` array.
    pub fn parse_from_json(included_value: &Value) -> Result<Self, Error> {
        let mut included = Self {
            resources: Vec::new(),
            entitlements: Vec::new(),
            components: Vec::new(),
            groups: Vec::new(),
        };

        if let Some(included_array) = included_value.as_array() {
            for item in included_array {
                let resource: IncludedResource = serde_json::from_value(item.clone())
                    .map_err(|error| Error::LicenseFileInvalid(error.to_string()))?;
                included.resources.push(resource);

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
                            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                if let Some(attrs) = item.get("attributes") {
                                    if let (Some(fingerprint), Some(name), metadata) = (
                                        attrs.get("fingerprint").and_then(|f| f.as_str()),
                                        attrs.get("name").and_then(|n| n.as_str()),
                                        attrs
                                            .get("metadata")
                                            .and_then(|m| serde_json::from_value(m.clone()).ok()),
                                    ) {
                                        included.components.push(Component {
                                            id: id.to_string(),
                                            fingerprint: fingerprint.to_string(),
                                            name: name.to_string(),
                                            metadata,
                                            ..Default::default()
                                        });
                                    }
                                }
                            }
                        }
                        "groups" => {
                            if let Some(id) = item.get("id").and_then(|i| i.as_str()) {
                                if let Some(attrs) = item.get("attributes") {
                                    included.groups.push(Group {
                                        id: id.to_string(),
                                        name: attrs
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .unwrap_or("Unknown Group")
                                            .to_string(),
                                        max_users: attrs
                                            .get("maxUsers")
                                            .and_then(|v| v.as_i64())
                                            .map(|v| v as i32),
                                        max_licenses: attrs
                                            .get("maxLicenses")
                                            .and_then(|v| v.as_i64())
                                            .map(|v| v as i32),
                                        max_machines: attrs
                                            .get("maxMachines")
                                            .and_then(|v| v.as_i64())
                                            .map(|v| v as i32),
                                        metadata: attrs
                                            .get("metadata")
                                            .and_then(|m| m.as_object())
                                            .map(|obj| {
                                                obj.iter()
                                                    .map(|(k, v)| (k.clone(), v.clone()))
                                                    .collect()
                                            }),
                                        created: attrs
                                            .get("created")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.parse().ok())
                                            .unwrap_or_else(Utc::now),
                                        updated: attrs
                                            .get("updated")
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.parse().ok())
                                            .unwrap_or_else(Utc::now),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(included)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct LicenseFileDataset {
    pub license: License,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
    #[serde(default)]
    pub included: Option<IncludedResources>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
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

    /// Load an encrypted or unencrypted certificate without verifying its signature.
    ///
    /// `key` is used only to decrypt encrypted certificates. Prefer
    /// [`Self::from_cert_verified`] when restoring a license from untrusted storage.
    pub fn from_cert(key: &str, content: &str) -> Result<LicenseFile, Error> {
        let certificate = Self::_certificate(content.to_string())?;
        let dataset = Self::_unpack(key, &certificate)?;
        Ok(Self::_from_dataset(content, dataset))
    }

    /// Verify and load a certificate for the expected license key.
    ///
    /// The signature is verified before decoding or decrypting the dataset. An
    /// expired dataset is returned in [`Error::LicenseFileExpired`] only after
    /// its signature and license key have both been checked.
    ///
    /// Verification uses the certificate's declared algorithm and an explicit
    /// public key. `key` must match the dataset's license key and is also used
    /// to decrypt encrypted certificates.
    pub fn from_cert_verified(
        key: &str,
        content: &str,
        public_key: &str,
    ) -> Result<LicenseFile, Error> {
        let certificate = Self::_certificate(content.to_string())?;
        Verifier::new(public_key.to_string()).verify_license_certificate(&certificate)?;
        let (dataset, expired) = match Self::_unpack(key, &certificate) {
            Ok(dataset) => (dataset, false),
            Err(Error::LicenseFileExpired(dataset)) => (*dataset, true),
            Err(error) => return Err(error),
        };
        if dataset.license.key != key {
            return Err(Error::LicenseFileInvalid(
                "Certificate license key does not match the expected key".to_string(),
            ));
        }
        if expired {
            return Err(Error::LicenseFileExpired(Box::new(dataset)));
        }
        Ok(Self::_from_dataset(content, dataset))
    }

    fn _from_dataset(content: &str, dataset: LicenseFileDataset) -> LicenseFile {
        LicenseFile {
            id: dataset.license.id,
            certificate: content.to_string(),
            issued: dataset.issued,
            expiry: dataset.expiry,
            ttl: dataset.ttl,
        }
    }

    /// Verify using the globally configured public key and certificate algorithm.
    ///
    /// Prefer [`Self::verify_with_key_and_algorithm`] for untrusted input so the
    /// certificate cannot select an unexpected algorithm.
    pub fn verify(&self) -> Result<(), Error> {
        let config = crate::config::get_config()?;

        if let Some(public_key) = &config.public_key {
            let verifier = Verifier::new(public_key.clone());
            verifier.verify_license_file(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    /// Decrypt without verifying the certificate signature.
    ///
    /// Prefer [`Self::verify_and_decrypt`] when consuming untrusted input.
    pub fn decrypt(&self, key: &str) -> Result<LicenseFileDataset, Error> {
        Self::_decrypt(key, &self.certificate)
    }

    /// Decode an unencrypted base64 license file without verifying its signature.
    ///
    /// Prefer [`Self::verify_and_decode`] when consuming untrusted input.
    pub fn decode(&self) -> Result<LicenseFileDataset, Error> {
        Self::_decode(&self.certificate)
    }

    /// Verify with an explicit public key and the certificate's declared algorithm.
    ///
    /// Prefer [`Self::verify_with_key_and_algorithm`] for untrusted input.
    pub fn verify_with_key(&self, public_key: &str) -> Result<(), Error> {
        Verifier::new(public_key.to_string()).verify_license_file(self)
    }

    /// Verify only when the certificate uses the caller's expected algorithm.
    pub fn verify_with_key_and_algorithm(
        &self,
        public_key: &str,
        expected_algorithm: LicenseFileAlgorithm,
    ) -> Result<(), Error> {
        let actual_algorithm = self.algorithm()?;
        if actual_algorithm != expected_algorithm {
            return Err(Error::LicenseFileAlgorithmMismatch {
                expected: expected_algorithm.as_str().to_string(),
                actual: actual_algorithm.as_str().to_string(),
            });
        }
        self.verify_with_key(public_key)
    }

    /// Return the algorithm declared by this license file.
    pub fn algorithm(&self) -> Result<LicenseFileAlgorithm, Error> {
        let certificate = self.certificate()?;
        LicenseFileAlgorithm::from_code(&certificate.alg)
            .ok_or(Error::LicenseFileNotSupported(certificate.alg))
    }

    /// Verify and decode an unencrypted license file in one operation.
    pub fn verify_and_decode(
        &self,
        public_key: &str,
        expected_algorithm: LicenseFileAlgorithm,
    ) -> Result<LicenseFileDataset, Error> {
        self.verify_with_key_and_algorithm(public_key, expected_algorithm)?;
        self.decode()
    }

    /// Verify and decrypt an encrypted license file in one operation.
    pub fn verify_and_decrypt(
        &self,
        public_key: &str,
        decryption_key: &str,
        expected_algorithm: LicenseFileAlgorithm,
    ) -> Result<LicenseFileDataset, Error> {
        self.verify_with_key_and_algorithm(public_key, expected_algorithm)?;
        self.decrypt(decryption_key)
    }

    pub fn certificate(&self) -> Result<Certificate, Error> {
        Self::_certificate(self.certificate.clone())
    }

    /// Get entitlements from the license file without making an API call
    /// Requires the decryption key and the license file to include entitlements
    pub fn entitlements(&self, key: &str) -> Result<Vec<Entitlement>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_entitlements().cloned().unwrap_or_default())
    }

    /// Get components from the license file without making an API call
    /// Requires the decryption key and the license file to include components
    pub fn components(&self, key: &str) -> Result<Vec<Component>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_components().cloned().unwrap_or_default())
    }

    /// Get groups from the license file without making an API call
    /// Requires the decryption key and the license file to include groups
    pub fn groups(&self, key: &str) -> Result<Vec<Group>, Error> {
        let dataset = self.decrypt(key)?;
        Ok(dataset.offline_groups().cloned().unwrap_or_default())
    }

    fn _decrypt(key: &str, content: &str) -> Result<LicenseFileDataset, Error> {
        let cert = Self::_certificate(content.to_string())?;
        let algorithm = LicenseFileAlgorithm::from_code(&cert.alg)
            .ok_or_else(|| Error::LicenseFileNotSupported(cert.alg.clone()))?;
        if !algorithm.is_encrypted() {
            return Err(Error::LicenseFileNotEncrypted);
        }

        let decryptor = Decryptor::new(key.to_string());
        let data = decryptor.decrypt_certificate(&cert)?;
        Self::_parse_dataset(&data)
    }

    fn _unpack(key: &str, cert: &Certificate) -> Result<LicenseFileDataset, Error> {
        let algorithm = LicenseFileAlgorithm::from_code(&cert.alg)
            .ok_or_else(|| Error::LicenseFileNotSupported(cert.alg.clone()))?;
        let data = if algorithm.is_encrypted() {
            Decryptor::new(key.to_string()).decrypt_certificate(cert)?
        } else {
            general_purpose::STANDARD
                .decode(&cert.enc)
                .map_err(|error| Error::LicenseFileInvalid(error.to_string()))?
        };
        Self::_parse_dataset(&data)
    }

    fn _decode(content: &str) -> Result<LicenseFileDataset, Error> {
        let cert = Self::_certificate(content.to_string())?;
        let algorithm = LicenseFileAlgorithm::from_code(&cert.alg)
            .ok_or_else(|| Error::LicenseFileNotSupported(cert.alg.clone()))?;
        if algorithm.is_encrypted() {
            return Err(Error::LicenseFileNotSupported(cert.alg));
        }

        let data = general_purpose::STANDARD
            .decode(&cert.enc)
            .map_err(|error| Error::LicenseFileInvalid(error.to_string()))?;
        Self::_parse_dataset(&data)
    }

    fn _parse_dataset(data: &[u8]) -> Result<LicenseFileDataset, Error> {
        let dataset: Value =
            serde_json::from_slice(data).map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;

        let meta: CertificateFileMeta = serde_json::from_value(dataset["meta"].clone())
            .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;

        let data: KeygenResponseData<LicenseAttributes> =
            serde_json::from_value(dataset["data"].clone())
                .map_err(|e| Error::LicenseFileInvalid(e.to_string()))?;
        let license = License::from(data);

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
    use crate::license::{LicenseCheckoutOpts, UpdateField};
    use serde_json::json;

    const TEST_LICENSE_KEY: &str = "TEST-LICENSE-KEY";

    fn encode_certificate(certificate: &Certificate) -> String {
        let encoded = general_purpose::STANDARD.encode(serde_json::to_vec(certificate).unwrap());
        format!("-----BEGIN LICENSE FILE-----\n{encoded}\n-----END LICENSE FILE-----")
    }

    fn signed_certificate(
        encrypted: bool,
        expired: bool,
        license_key: &str,
    ) -> (Certificate, String) {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use ed25519_dalek::{Signer, SigningKey};
        use sha2::{Digest, Sha256};

        let dataset = json!({
            "meta": {
                "issued": "2020-01-01T00:00:00Z",
                "expiry": if expired { "2020-01-01T01:00:00Z" } else { "2999-01-01T00:00:00Z" },
                "ttl": 3600
            },
            "data": {
                "type": "licenses",
                "id": "license-1",
                "attributes": { "key": license_key },
                "relationships": {}
            },
            "included": [{
                "type": "entitlements",
                "id": "entitlement-1",
                "attributes": {
                    "code": "feature-a",
                    "created": "2020-01-01T00:00:00Z",
                    "updated": "2020-01-01T00:00:00Z"
                },
                "relationships": {}
            }]
        });
        let payload = serde_json::to_vec(&dataset).unwrap();
        let (enc, alg) = if encrypted {
            let cipher =
                Aes256Gcm::new_from_slice(&Sha256::digest(TEST_LICENSE_KEY.as_bytes())).unwrap();
            let iv = *b"012345678901";
            let ciphertext = cipher
                .encrypt(Nonce::from_slice(&iv), payload.as_ref())
                .unwrap();
            let (ciphertext, tag) = ciphertext.split_at(ciphertext.len() - 16);
            (
                format!(
                    "{}.{}.{}",
                    general_purpose::STANDARD.encode(ciphertext),
                    general_purpose::STANDARD.encode(iv),
                    general_purpose::STANDARD.encode(tag)
                ),
                "aes-256-gcm+ed25519",
            )
        } else {
            (general_purpose::STANDARD.encode(payload), "base64+ed25519")
        };
        let signing_key = SigningKey::from_bytes(&[7; 32]);
        let signature = signing_key.sign(format!("license/{enc}").as_bytes());
        (
            Certificate {
                enc,
                sig: general_purpose::STANDARD.encode(signature.to_bytes()),
                alg: alg.to_string(),
            },
            hex::encode(signing_key.verifying_key().to_bytes()),
        )
    }

    #[test]
    fn verified_loading_preserves_metadata_and_entitlements_for_both_encodings() {
        for encrypted in [false, true] {
            let (certificate, public_key) = signed_certificate(encrypted, false, TEST_LICENSE_KEY);
            let content = encode_certificate(&certificate);
            let file =
                LicenseFile::from_cert_verified(TEST_LICENSE_KEY, &content, &public_key).unwrap();
            assert_eq!(file.id, "license-1");
            assert_eq!(file.certificate, content);
            assert_eq!(file.ttl, 3600);
            let dataset = if encrypted {
                assert!(file.decode().is_err());
                file.decrypt(TEST_LICENSE_KEY).unwrap()
            } else {
                assert!(matches!(
                    file.decrypt(TEST_LICENSE_KEY),
                    Err(Error::LicenseFileNotEncrypted)
                ));
                file.decode().unwrap()
            };
            assert_eq!(file.issued, dataset.issued);
            assert_eq!(file.expiry, dataset.expiry);
            assert_eq!(dataset.license.key, TEST_LICENSE_KEY);
            assert_eq!(dataset.offline_entitlements().unwrap()[0].code, "feature-a");
        }
    }

    #[test]
    fn verified_loading_rejects_tampering_before_unpacking_or_returning_expired_data() {
        for encrypted in [false, true] {
            for expired in [false, true] {
                let (certificate, public_key) =
                    signed_certificate(encrypted, expired, TEST_LICENSE_KEY);
                let mut tampered_signature = certificate.clone();
                tampered_signature.sig = general_purpose::STANDARD.encode([0; 64]);
                let mut tampered_payload = certificate;
                // Invalid payload also checks that verification precedes decoding/decryption.
                tampered_payload.enc = "tampered".to_string();
                for tampered in [tampered_signature, tampered_payload] {
                    assert!(matches!(
                        LicenseFile::from_cert_verified(
                            TEST_LICENSE_KEY,
                            &encode_certificate(&tampered),
                            &public_key
                        ),
                        Err(Error::LicenseFileNotGenuine(_))
                    ));
                }
            }
        }
    }

    #[test]
    fn verified_loading_rejects_wrong_verification_and_decryption_keys() {
        let other_public_key = hex::encode(
            ed25519_dalek::SigningKey::from_bytes(&[8; 32])
                .verifying_key()
                .to_bytes(),
        );
        for encrypted in [false, true] {
            let (certificate, public_key) = signed_certificate(encrypted, false, TEST_LICENSE_KEY);
            let content = encode_certificate(&certificate);
            assert!(matches!(
                LicenseFile::from_cert_verified(TEST_LICENSE_KEY, &content, &other_public_key),
                Err(Error::LicenseFileNotGenuine(_))
            ));
            assert!(matches!(
                LicenseFile::from_cert_verified(TEST_LICENSE_KEY, &content, ""),
                Err(Error::PublicKeyMissing)
            ));
            let result = LicenseFile::from_cert_verified("wrong-key", &content, &public_key);
            if encrypted {
                assert!(matches!(result, Err(Error::DecryptionError(_))));
            } else {
                assert!(matches!(result, Err(Error::LicenseFileInvalid(_))));
            }
        }
    }

    #[test]
    fn verified_loading_checks_dataset_key_before_exposing_valid_or_expired_data() {
        for encrypted in [false, true] {
            for expired in [false, true] {
                // Encryption still uses TEST_LICENSE_KEY, so even encrypted files
                // must check the identity inside the signed dataset.
                let (certificate, public_key) =
                    signed_certificate(encrypted, expired, "OTHER-LICENSE-KEY");
                assert!(matches!(
                    LicenseFile::from_cert_verified(
                        TEST_LICENSE_KEY,
                        &encode_certificate(&certificate),
                        &public_key
                    ),
                    Err(Error::LicenseFileInvalid(_))
                ));
            }
        }
    }

    #[test]
    fn loading_preserves_expired_datasets_for_both_encodings() {
        for encrypted in [false, true] {
            let (certificate, public_key) = signed_certificate(encrypted, true, TEST_LICENSE_KEY);
            let content = encode_certificate(&certificate);
            for result in [
                LicenseFile::from_cert(TEST_LICENSE_KEY, &content),
                LicenseFile::from_cert_verified(TEST_LICENSE_KEY, &content, &public_key),
            ] {
                match result {
                    Err(Error::LicenseFileExpired(dataset)) => {
                        assert_eq!(dataset.license.key, TEST_LICENSE_KEY);
                        assert_eq!(dataset.ttl, 3600);
                        assert_eq!(dataset.offline_entitlements().unwrap()[0].code, "feature-a");
                    }
                    result => panic!("Expected expired dataset, got {result:?}"),
                }
            }
        }
    }

    #[test]
    fn loading_rejects_malformed_and_unsupported_certificates() {
        let (mut certificate, public_key) = signed_certificate(false, false, TEST_LICENSE_KEY);
        certificate.alg = "base64+unknown".to_string();
        for content in ["invalid".to_string(), encode_certificate(&certificate)] {
            for result in [
                LicenseFile::from_cert(TEST_LICENSE_KEY, &content),
                LicenseFile::from_cert_verified(TEST_LICENSE_KEY, &content, &public_key),
            ] {
                if content == "invalid" {
                    assert!(matches!(result, Err(Error::LicenseFileInvalid(_))));
                } else {
                    assert!(matches!(result, Err(Error::LicenseFileNotSupported(_))));
                }
            }
        }
    }

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
        assert_eq!(included.resources.len(), 2);
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
                ..Default::default()
            }],
            groups: vec![],
            resources: vec![],
        };

        let dataset = LicenseFileDataset {
            license: {
                let mut license = License::from_id("lic1");
                license.key = "test-key".to_string();
                license.name = Some("Test License".to_string());
                license.status = Some("active".to_string());
                license.uses = Some(0);
                license.max_machines = Some(5);
                license.protected = Some(false);
                license.suspended = Some(false);
                license.policy = Some("policy1".to_string());
                license.account_id = Some("acc1".to_string());
                license.product_id = Some("prod1".to_string());
                license
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

        assert_eq!(opts.ttl, UpdateField::Set(7200));
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
        assert_eq!(opts.ttl, UpdateField::Keep);
    }

    #[test]
    fn test_license_checkout_opts_new() {
        let opts = LicenseCheckoutOpts::new();

        assert_eq!(opts.ttl, UpdateField::Keep);
        assert!(opts.include.is_none());
    }

    #[test]
    fn decodes_base64_license_files_and_preserves_all_included_resources() {
        let dataset = json!({
            "meta": {
                "issued": "2026-01-01T00:00:00Z",
                "expiry": "2026-01-01T00:00:00Z",
                "ttl": 0
            },
            "data": {
                "type": "licenses",
                "id": "license-1",
                "attributes": {
                    "key": "TEST-LICENSE-KEY"
                },
                "relationships": {}
            },
            "included": [
                {
                    "type": "products",
                    "id": "product-1",
                    "attributes": {
                        "name": "Desktop"
                    },
                    "relationships": {
                        "account": {
                            "data": { "type": "accounts", "id": "account-1" }
                        }
                    },
                    "links": {
                        "self": "/v1/products/product-1"
                    }
                }
            ]
        });
        let encoded_dataset =
            general_purpose::STANDARD.encode(serde_json::to_vec(&dataset).unwrap());
        for algorithm in [
            "base64+ed25519",
            "base64+ecdsa-p256",
            "base64+rsa-pss-sha256",
            "base64+rsa-sha256",
        ] {
            let certificate = Certificate {
                enc: encoded_dataset.clone(),
                sig: String::new(),
                alg: algorithm.to_string(),
            };
            let encoded_certificate =
                general_purpose::STANDARD.encode(serde_json::to_vec(&certificate).unwrap());
            let file = LicenseFile {
                id: "file-1".to_string(),
                certificate: format!(
                    "-----BEGIN LICENSE FILE-----\n{encoded_certificate}\n-----END LICENSE FILE-----"
                ),
                issued: "2026-01-01T00:00:00Z".parse().unwrap(),
                expiry: "2026-01-01T00:00:00Z".parse().unwrap(),
                ttl: 0,
            };

            assert_eq!(
                file.algorithm().unwrap(),
                LicenseFileAlgorithm::from_code(algorithm).unwrap()
            );
            let decoded = file.decode().unwrap();
            assert_eq!(
                LicenseFile::from_cert("unused", &file.certificate)
                    .unwrap()
                    .id,
                "license-1"
            );
            assert_eq!(decoded.license.id, "license-1");
            let included = decoded.included.unwrap();
            let products = included.resources_by_type("products");
            assert_eq!(products.len(), 1);
            assert_eq!(products[0].attributes["name"], "Desktop");
            assert_eq!(
                products[0].relationships["account"]["data"]["id"],
                "account-1"
            );
            assert_eq!(products[0].links["self"], "/v1/products/product-1");
        }
    }

    #[test]
    fn rejects_an_unexpected_license_file_algorithm_before_verification() {
        let certificate = Certificate {
            enc: "dataset".to_string(),
            sig: String::new(),
            alg: "base64+ed25519".to_string(),
        };
        let encoded_certificate =
            general_purpose::STANDARD.encode(serde_json::to_vec(&certificate).unwrap());
        let file = LicenseFile {
            id: "file-1".to_string(),
            certificate: format!(
                "-----BEGIN LICENSE FILE-----\n{encoded_certificate}\n-----END LICENSE FILE-----"
            ),
            issued: "2026-01-01T00:00:00Z".parse().unwrap(),
            expiry: "2026-01-01T00:00:00Z".parse().unwrap(),
            ttl: 0,
        };

        assert!(matches!(
            file.verify_with_key_and_algorithm("", LicenseFileAlgorithm::Base64EcdsaP256),
            Err(Error::LicenseFileAlgorithmMismatch { .. })
        ));
    }

    #[test]
    fn decrypts_every_aes_license_file_algorithm() {
        use aes_gcm::aead::Aead;
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use sha2::{Digest, Sha256};

        let secret = "checkout-secret";
        let dataset = json!({
            "meta": {
                "issued": "2026-01-01T00:00:00Z",
                "expiry": "2026-01-01T00:00:00Z",
                "ttl": 0
            },
            "data": {
                "type": "licenses",
                "id": "license-1",
                "attributes": { "key": "TEST-LICENSE-KEY" },
                "relationships": {}
            }
        });
        let cipher = Aes256Gcm::new_from_slice(&Sha256::digest(secret.as_bytes())).unwrap();
        let iv = *b"012345678901";
        let encrypted = cipher
            .encrypt(
                Nonce::from_slice(&iv),
                serde_json::to_vec(&dataset).unwrap().as_ref(),
            )
            .unwrap();
        let (ciphertext, tag) = encrypted.split_at(encrypted.len() - 16);
        let encrypted_dataset = format!(
            "{}.{}.{}",
            general_purpose::STANDARD.encode(ciphertext),
            general_purpose::STANDARD.encode(iv),
            general_purpose::STANDARD.encode(tag)
        );

        for algorithm in [
            "aes-256-gcm+ed25519",
            "aes-256-gcm+ecdsa-p256",
            "aes-256-gcm+rsa-pss-sha256",
            "aes-256-gcm+rsa-sha256",
        ] {
            let certificate = Certificate {
                enc: encrypted_dataset.clone(),
                sig: String::new(),
                alg: algorithm.to_string(),
            };
            let encoded_certificate =
                general_purpose::STANDARD.encode(serde_json::to_vec(&certificate).unwrap());
            let file = LicenseFile {
                id: "file-1".to_string(),
                certificate: format!(
                    "-----BEGIN LICENSE FILE-----\n{encoded_certificate}\n-----END LICENSE FILE-----"
                ),
                issued: "2026-01-01T00:00:00Z".parse().unwrap(),
                expiry: "2026-01-01T00:00:00Z".parse().unwrap(),
                ttl: 0,
            };

            assert_eq!(file.decrypt(secret).unwrap().license.id, "license-1");
            assert_eq!(
                LicenseFile::from_cert(secret, &file.certificate)
                    .unwrap()
                    .id,
                "license-1"
            );
        }
    }
}
