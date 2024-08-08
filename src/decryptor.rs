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
        unimplemented!()
    }
}
