use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::component::Component;
use crate::entitlement::Entitlement;
use crate::errors::Error;
use crate::license_file::LicenseFile;
use crate::machine::Machine;
use crate::verifier::Verifier;
use crate::PUBLIC_KEY;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemeCode {
    #[serde(rename = "ED25519_SIGN")]
    Ed25519Sign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub key: String,
    pub expiry: Option<DateTime<Utc>>,
    pub scheme: Option<SchemeCode>,
    pub require_heartbeat: bool,
    pub last_validated: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl License {
    pub async fn validate(&self, fingerprints: &[String]) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn verify(&self) -> Result<Vec<u8>, Error> {
        if self.scheme.is_none() {
            return Err(Error::LicenseNotSigned);
        }
        if PUBLIC_KEY.is_none() {
            return Err(Error::PublicKeyMissing);
        }
        if let Some(public_key) = PUBLIC_KEY.clone() {
            let verifier = Verifier::new(public_key);
            verifier.verify_license(self)
        } else {
            Err(Error::PublicKeyMissing)
        }
    }

    pub async fn activate(
        &self,
        fingerprint: &str,
        components: &[Component],
    ) -> Result<Machine, Error> {
        unimplemented!()
    }

    pub async fn deactivate(&self, id: &str) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn machine(&self, id: &str) -> Result<Machine, Error> {
        unimplemented!()
    }

    pub async fn machines(&self) -> Result<Vec<Machine>, Error> {
        unimplemented!()
    }

    pub async fn entitlements(&self) -> Result<Vec<Entitlement>, Error> {
        unimplemented!()
    }

    pub async fn checkout(&self, options: &CheckoutOptions) -> Result<LicenseFile, Error> {
        unimplemented!()
    }
}

pub struct CheckoutOptions {
    // Define checkout options here
}
