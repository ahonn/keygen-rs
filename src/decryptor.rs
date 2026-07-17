//! Decryption for encrypted license and machine files.
//!
//! This module provides AES-256-GCM decryption for license files and machine files
//! that have been encrypted by the Keygen API.

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use base64::engine::general_purpose;
use base64::Engine;
use sha2::Digest;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::certificate::Certificate;
use crate::errors::Error;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Decryptor {
    secret: String,
}

impl Decryptor {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn decrypt_certificate(&self, cert: &Certificate) -> Result<Vec<u8>, Error> {
        let parts: Vec<&str> = cert.enc.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::DecryptionError(
                "Invalid encrypted data format".into(),
            ));
        }

        let ciphertext = general_purpose::STANDARD
            .decode(parts[0])
            .map_err(|_| Error::DecryptionError("Failed to decode ciphertext".into()))?;
        let iv = general_purpose::STANDARD
            .decode(parts[1])
            .map_err(|_| Error::DecryptionError("Failed to decode IV".into()))?;
        let tag = general_purpose::STANDARD
            .decode(parts[2])
            .map_err(|_| Error::DecryptionError("Failed to decode tag".into()))?;

        if iv.len() != 12 {
            return Err(Error::DecryptionError(format!(
                "Invalid IV length: expected 12 bytes, got {}",
                iv.len()
            )));
        }
        if tag.len() != 16 {
            return Err(Error::DecryptionError(format!(
                "Invalid authentication tag length: expected 16 bytes, got {}",
                tag.len()
            )));
        }

        let mut hasher = Sha256::new();
        hasher.update(self.secret.as_bytes());
        let key = hasher.finalize();

        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|_| Error::DecryptionError("Invalid key length".into()))?;
        let nonce = Nonce::from_slice(&iv);

        let mut encrypted_data = ciphertext;
        encrypted_data.extend_from_slice(&tag);

        let plaintext = cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|_| Error::DecryptionError("Decryption failed".into()))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_iv_and_tag_lengths_without_panicking() {
        let decryptor = Decryptor::new("secret".to_string());
        let invalid_iv = Certificate {
            enc: "Y2lwaGVydGV4dA==.c2hvcnQ=.dGFnMDEyMzQ1Njc4OTAxMg==".to_string(),
            sig: String::new(),
            alg: "aes-256-gcm+ed25519".to_string(),
        };
        let invalid_tag = Certificate {
            enc: "Y2lwaGVydGV4dA==.MDEyMzQ1Njc4OTAx.c2hvcnQ=".to_string(),
            sig: String::new(),
            alg: "aes-256-gcm+ed25519".to_string(),
        };

        assert!(matches!(
            decryptor.decrypt_certificate(&invalid_iv),
            Err(Error::DecryptionError(_))
        ));
        assert!(matches!(
            decryptor.decrypt_certificate(&invalid_tag),
            Err(Error::DecryptionError(_))
        ));
    }
}
