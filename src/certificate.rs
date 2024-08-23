use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::KeygenResponseData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub enc: String,
    pub sig: String,
    pub alg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateFileMeta {
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateFileAttributes {
    pub certificate: String,
    pub issued: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub ttl: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CartificateFileResponse {
    pub data: KeygenResponseData<CertificateFileAttributes>,
}
