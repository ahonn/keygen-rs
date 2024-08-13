use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{errors::Error, KeygenResponseData};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFileAttributes {
    pub certificate: String,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LicenseFileResponse {
  pub data: KeygenResponseData<LicenseFileAttributes>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseFile {
    pub id: String,
    pub license_id: String,
    pub attributes: LicenseFileAttributes,
}

impl LicenseFile {
    pub fn verify(&self) -> Result<(), Error> {
        unimplemented!()
    }
}
