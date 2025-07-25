use base64::{engine::general_purpose, Engine};
use ed25519_dalek::{VerifyingKey, Signature, Verifier as Ed25519Verifier};
use reqwest::header::HeaderMap;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::certificate::Certificate;
use crate::errors::Error;
use crate::license::{License, SchemeCode};
use crate::license_file::LicenseFile;
use crate::machine_file::MachineFile;

#[allow(dead_code)]
struct SignatureComponents {
    keyid: String,
    algorithm: String,
    signature: Vec<u8>,
    headers: String,
}

pub struct Verifier {
    public_key: String,
}

impl Verifier {
    pub fn new(public_key: String) -> Self {
        Self { public_key }
    }

    pub fn verify_machine_file(&self, lic: &MachineFile) -> Result<(), Error> {
        let cert = lic.certificate()?;
        if let Err(e) = self.verify_certificate(&cert, "machine") {
            match e {
                Error::CertificateFileNotGenuine(msg) => {
                    return Err(Error::MachineFileNotGenuine(msg))
                }
                Error::CertificateFileNotSupported(msg) => {
                    return Err(Error::MachineFileNotSupported(msg))
                }
                _ => return Err(e),
            }
        }
        Ok(())
    }

    pub fn verify_license_file(&self, lic: &LicenseFile) -> Result<(), Error> {
        let cert = lic.certificate()?;
        if let Err(e) = self.verify_certificate(&cert, "license") {
            match e {
                Error::CertificateFileNotGenuine(msg) => {
                    return Err(Error::LicenseFileNotGenuine(msg))
                }
                Error::CertificateFileNotSupported(msg) => {
                    return Err(Error::LicenseFileNotSupported(msg))
                }
                _ => return Err(e),
            }
        }
        Ok(())
    }

    pub fn verify_license(&self, license: &License) -> Result<Vec<u8>, Error> {
        if license.key.is_empty() {
            return Err(Error::LicenseKeyMissing);
        }
        if license.scheme.is_none() {
            return Err(Error::LicenseSchemeMissing);
        }
        match license.scheme.as_ref().unwrap() {
            SchemeCode::Ed25519Sign => self.verify_key(&license.key),
            #[allow(unreachable_patterns)]
            _ => Err(Error::LicenseSchemeUnsupported),
        }
    }

    pub fn verify_keygen_signature(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        method: &str,
        path: &str,
        host: &str,
    ) -> Result<(), Error> {
        let signature_header = self.get_required_header(headers, "keygen-signature")?;
        let date_header = self.get_required_header(headers, "date")?;
        let digest_header = self.get_required_header(headers, "digest")?;

        self.verify_digest(digest_header, body)?;

        let signature_components = self.parse_signature_header(signature_header)?;
        if signature_components.algorithm != "ed25519" {
            return Err(Error::KeygenSignatureInvalid {
                reason: format!("Unsupported algorithm: {}", signature_components.algorithm),
            });
        }

        let request_target = format!("{} {}", method.to_lowercase(), path);
        let signing_data = format!(
            "(request-target): {}\nhost: {}\ndate: {}\ndigest: {}",
            request_target, host, date_header, digest_header
        );

        self.verify_ed25519_signature(&signing_data, &signature_components.signature)?;

        Ok(())
    }

    fn get_required_header<'a>(
        &self,
        headers: &'a HeaderMap,
        name: &str,
    ) -> Result<&'a str, Error> {
        headers
            .get(name)
            .ok_or_else(|| Error::KeygenSignatureInvalid {
                reason: format!("Missing {} header", name),
            })?
            .to_str()
            .map_err(|_| Error::KeygenSignatureInvalid {
                reason: format!("Invalid {} header", name),
            })
    }

    fn parse_signature_header(&self, signature_header: &str) -> Result<SignatureComponents, Error> {
        let mut keyid = String::new();
        let mut algorithm = String::new();
        let mut signature_b64 = String::new();
        let mut headers = String::new();

        for part in signature_header.split(',') {
            let part = part.trim();
            if let Some(value) = self.extract_header_value(part, "keyid=") {
                keyid = value;
            } else if let Some(value) = self.extract_header_value(part, "algorithm=") {
                algorithm = value;
            } else if let Some(value) = self.extract_header_value(part, "signature=") {
                signature_b64 = value;
            } else if let Some(value) = self.extract_header_value(part, "headers=") {
                headers = value;
            }
        }

        if keyid.is_empty()
            || algorithm.is_empty()
            || signature_b64.is_empty()
            || headers.is_empty()
        {
            return Err(Error::KeygenSignatureInvalid {
                reason: "Missing signature components".to_string(),
            });
        }

        let signature = general_purpose::STANDARD
            .decode(signature_b64)
            .map_err(|_| Error::KeygenSignatureInvalid {
                reason: "Invalid signature encoding".to_string(),
            })?;

        Ok(SignatureComponents {
            keyid,
            algorithm,
            signature,
            headers,
        })
    }

    fn extract_header_value<'a>(&self, part: &'a str, prefix: &str) -> Option<String> {
        if part.starts_with(prefix) {
            Some(
                part.trim_start_matches(prefix)
                    .trim_matches('"')
                    .to_string(),
            )
        } else {
            None
        }
    }

    fn verify_digest(&self, digest_header: &str, body: &[u8]) -> Result<(), Error> {
        const SHA256_PREFIX: &str = "sha-256=";

        if digest_header.starts_with(SHA256_PREFIX) {
            let provided_digest = &digest_header[SHA256_PREFIX.len()..];

            let mut hasher = Sha256::new();
            hasher.update(body);
            let digest_bytes = hasher.finalize();
            let calculated_digest = general_purpose::STANDARD.encode(digest_bytes);

            if !bool::from(provided_digest.as_bytes().ct_eq(calculated_digest.as_bytes())) {
                return Err(Error::KeygenSignatureInvalid {
                    reason: "Body digest does not match digest header".to_string(),
                });
            }
            Ok(())
        } else {
            Err(Error::KeygenSignatureInvalid {
                reason: "Unsupported digest algorithm".to_string(),
            })
        }
    }

    fn verify_ed25519_signature(
        &self,
        signing_data: &str,
        signature_bytes: &[u8],
    ) -> Result<(), Error> {
        let public_key_bytes = self.public_key_bytes()?;
        let public_key = VerifyingKey::from_bytes(&public_key_bytes).map_err(|_| {
            Error::KeygenSignatureInvalid {
                reason: "Invalid public key".to_string(),
            }
        })?;

        let signature =
            Signature::try_from(signature_bytes).map_err(|_| Error::KeygenSignatureInvalid {
                reason: "Invalid signature format".to_string(),
            })?;

        public_key
            .verify(signing_data.as_bytes(), &signature)
            .map_err(|_| Error::KeygenSignatureInvalid {
                reason: "Signature verification failed".to_string(),
            })?;

        Ok(())
    }

    fn verify_certificate(&self, cert: &Certificate, prefix: &str) -> Result<(), Error> {
        match cert.alg.as_str() {
            "aes-256-gcm+ed25519" | "base64+ed25519" => {
                let public_key = self.public_key_bytes()?;

                let msg = format!("{}/{}", prefix, cert.enc).into_bytes();
                let sig = general_purpose::STANDARD
                    .decode(&cert.sig)
                    .map_err(|e| Error::CertificateFileNotGenuine(e.to_string()))?;

                let public_key = VerifyingKey::from_bytes(&public_key)
                    .map_err(|e| Error::CertificateFileNotGenuine(e.to_string()))?;
                let signature = Signature::try_from(&sig[..])
                    .map_err(|e| Error::CertificateFileNotGenuine(e.to_string()))?;

                if let Err(e) = public_key.verify(&msg, &signature) {
                    return Err(Error::CertificateFileNotGenuine(e.to_string()));
                };
                Ok(())
            }
            _ => Err(Error::CertificateFileNotSupported(cert.alg.clone())),
        }
    }

    fn verify_key(&self, key: &str) -> Result<Vec<u8>, Error> {
        let public_key = self.public_key_bytes()?;

        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() != 2 {
            return Err(Error::LicenseKeyNotGenuine);
        }

        let signing_data = parts[0];
        let enc_sig = parts[1];

        let parts: Vec<&str> = signing_data.split('/').collect();
        if parts.len() != 2 || parts[0] != "key" {
            return Err(Error::LicenseKeyNotGenuine);
        }

        let enc_dataset = parts[1];

        let msg = format!("key/{}", enc_dataset).into_bytes();
        let sig = general_purpose::URL_SAFE
            .decode(enc_sig)
            .map_err(|_| Error::LicenseKeyNotGenuine)?;

        let dataset = general_purpose::URL_SAFE
            .decode(enc_dataset)
            .map_err(|_| Error::LicenseKeyNotGenuine)?;

        let public_key = VerifyingKey::from_bytes(&public_key).map_err(|_| Error::PublicKeyInvalid)?;
        let signature = Signature::try_from(&sig[..]).map_err(|_| Error::LicenseKeyNotGenuine)?;

        if public_key.verify(&msg, &signature).is_ok() {
            Ok(dataset)
        } else {
            Err(Error::LicenseKeyNotGenuine)
        }
    }

    fn public_key_bytes(&self) -> Result<[u8; 32], Error> {
        if self.public_key.is_empty() {
            return Err(Error::PublicKeyMissing);
        }

        let key = hex::decode(&self.public_key).map_err(|_| Error::PublicKeyInvalid)?;
        if key.len() != 32 {
            return Err(Error::PublicKeyInvalid);
        }

        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key);
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::license::SchemeCode;
    use base64::engine::general_purpose;
    use ed25519_dalek::{SigningKey, Signer};
    use rand::rngs::OsRng;
    use reqwest::header::{HeaderMap, HeaderValue};
    use serde_json::json;

    fn generate_valid_keys() -> (String, String) {
        let mut csprng = OsRng::default();
        let keypair: SigningKey = SigningKey::generate(&mut csprng);

        let public_key = hex::encode(keypair.verifying_key().as_bytes());

        let payload = json!({
          "lic": "TEST-LICENSE-KEY",
          "exp": "2025-12-31",
          "iss": "keygen",
        });

        let payload_encoded = general_purpose::URL_SAFE.encode(payload.to_string());

        let signing_input = format!("key/{}", payload_encoded);
        let signature = keypair.sign(signing_input.as_bytes());

        let license_key = format!(
            "{}.{}",
            signing_input,
            general_purpose::URL_SAFE.encode(signature.to_bytes())
        );

        (public_key, license_key)
    }

    fn create_test_license(key: &str) -> License {
        License {
            id: String::new(),
            scheme: Some(SchemeCode::Ed25519Sign),
            name: Some("Test License".to_string()),
            key: key.to_string(),
            expiry: None,
            status: None,
            uses: None,
            max_machines: None,
            max_cores: None,
            max_uses: None,
            max_processes: None,
            max_users: None,
            protected: None,
            suspended: None,
            permissions: None,
            policy: None,
            metadata: HashMap::new(),
            account_id: None,
            product_id: None,
            group_id: None,
            owner_id: None,
        }
    }

    #[test]
    fn test_verify_license() {
        let (public_key, license_key) = generate_valid_keys();
        let verifier = Verifier::new(public_key);
        let license = create_test_license(&license_key);

        let result = verifier.verify_license(&license);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_license_with_missing_key() {
        let (public_key, _) = generate_valid_keys();
        let verifier = Verifier::new(public_key);
        let mut license = create_test_license("");
        license.key = String::new();

        let result = verifier.verify_license(&license);
        assert!(matches!(result, Err(Error::LicenseKeyMissing)));
    }

    #[test]
    fn test_verify_license_with_missing_scheme() {
        let (public_key, license_key) = generate_valid_keys();
        let verifier = Verifier::new(public_key);
        let mut license = create_test_license(&license_key);
        license.scheme = None;

        let result = verifier.verify_license(&license);
        assert!(matches!(result, Err(Error::LicenseSchemeMissing)));
    }

    #[test]
    fn test_verify_license_with_invalid_key() {
        let (public_key, _) = generate_valid_keys();
        let verifier = Verifier::new(public_key);
        let license = create_test_license("invalid.license.key");

        let result = verifier.verify_license(&license);
        assert!(matches!(result, Err(Error::LicenseKeyNotGenuine)));
    }

    #[test]
    fn test_verify_license_with_invalid_public_key() {
        let (_, license_key) = generate_valid_keys();
        let verifier = Verifier::new("invalid_public_key".to_string());
        let license = create_test_license(&license_key);

        let result = verifier.verify_license(&license);
        assert!(matches!(result, Err(Error::PublicKeyInvalid)));
    }

    #[test]
    fn test_verify_license_with_missing_public_key() {
        let (_, license_key) = generate_valid_keys();
        let verifier = Verifier::new("".to_string());
        let license = create_test_license(&license_key);

        let result = verifier.verify_license(&license);
        assert!(matches!(result, Err(Error::PublicKeyMissing)));
    }

    #[test]
    fn test_verify_keygen_signature() {
        // Generate keypair for testing
        let mut csprng = OsRng::default();
        let keypair: SigningKey = SigningKey::generate(&mut csprng);
        let public_key = hex::encode(keypair.verifying_key().as_bytes());

        // Create a test body
        let body = b"test body";

        // Calculate the SHA-256 digest of the body
        let mut hasher = Sha256::new();
        hasher.update(body);
        let digest_bytes = hasher.finalize();
        let encoded_digest = general_purpose::STANDARD.encode(digest_bytes);
        let digest_header = format!("sha-256={}", encoded_digest);

        // Create date header
        let date = "Wed, 09 Jun 2021 16:08:15 GMT";

        // Create the signing data
        let request_target = "get /v1/accounts/keygen/licenses?limit=1";
        let host = "api.keygen.sh";

        let signing_data = format!(
            "(request-target): {}\nhost: {}\ndate: {}\ndigest: {}",
            request_target, host, date, digest_header
        );

        // Sign the data
        let signature = keypair.sign(signing_data.as_bytes());
        let signature_b64 = general_purpose::STANDARD.encode(signature.to_bytes());

        // Create the signature header
        let signature_header = format!(
            r#"keyid="test-account", algorithm="ed25519", signature="{}", headers="(request-target) host date digest""#,
            signature_b64
        );

        // Create the headers map
        let mut headers = HeaderMap::new();
        headers.insert(
            "keygen-signature",
            HeaderValue::from_str(&signature_header).unwrap(),
        );
        headers.insert("date", HeaderValue::from_str(date).unwrap());
        headers.insert("digest", HeaderValue::from_str(&digest_header).unwrap());

        let verifier = Verifier::new(public_key);
        let result = verifier.verify_keygen_signature(
            &headers,
            body,
            "GET",
            "/v1/accounts/keygen/licenses?limit=1",
            "api.keygen.sh",
        );
        assert!(
            result.is_ok(),
            "Signature verification should succeed: {:?}",
            result
        );
    }

    #[test]
    fn test_verify_keygen_signature_with_missing_header() {
        let mut csprng = OsRng::default();
        let keypair: SigningKey = SigningKey::generate(&mut csprng);
        let public_key = hex::encode(keypair.verifying_key().as_bytes());

        let body = b"test body";
        let headers = HeaderMap::new();

        let verifier = Verifier::new(public_key);
        let result = verifier.verify_keygen_signature(
            &headers,
            body,
            "GET",
            "/v1/accounts/keygen/licenses?limit=1",
            "api.keygen.sh",
        );
        assert!(matches!(result, Err(Error::KeygenSignatureInvalid { .. })));
    }
}
