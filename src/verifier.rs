use crate::errors::Error;
use crate::license::{License, SchemeCode};
use crate::license_file::LicenseFile;
use crate::machine_file::MachineFile;
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{PublicKey, Signature, Verifier as Ed25519Verifier};

pub struct Verifier {
    public_key: String,
}

impl Verifier {
    pub fn new(public_key: String) -> Self {
        Self { public_key }
    }

    pub fn verify_license_file(&self, license_file: &LicenseFile) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn verify_machine_file(&self, machine_file: &MachineFile) -> Result<(), Error> {
        unimplemented!()
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

        let public_key = PublicKey::from_bytes(&public_key).map_err(|_| Error::PublicKeyInvalid)?;
        let signature = Signature::from_bytes(&sig).map_err(|_| Error::LicenseKeyNotGenuine)?;

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

    pub fn verify_request(&self, request: &reqwest::Request) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn verify_response(&self, response: &reqwest::Response) -> Result<(), Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::license::SchemeCode;
    use base64::engine::general_purpose;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;
    use serde_json::json;

    fn generate_valid_keys() -> (String, String) {
        let mut csprng = OsRng::default();
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let public_key = hex::encode(keypair.public.as_bytes());

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
}
