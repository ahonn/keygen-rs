use aes_gcm::aead::Aead;
use aes_gcm::NewAead;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose;
use base64::Engine;
use sha2::Digest;
use sha2::Sha256;

use crate::certificate::Certificate;
use crate::errors::Error;

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

        let mut hasher = Sha256::new();
        hasher.update(self.secret.as_bytes());
        let key = hasher.finalize();

        let cipher = Aes256Gcm::new(Key::from_slice(&key));
        let nonce = Nonce::from_slice(&iv);

        let mut encrypted_data = ciphertext;
        encrypted_data.extend_from_slice(&tag);

        let plaintext = cipher
            .decrypt(nonce, encrypted_data.as_ref())
            .map_err(|_| Error::DecryptionError("Decryption failed".into()))?;

        Ok(plaintext)
    }
}
