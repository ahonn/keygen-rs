use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{errors::Error, KeygenResponseData};

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

pub fn validate_certificate_meta(meta: &CertificateFileMeta) -> Result<(), Error> {
    // let config = crate::config::get_config();

    // if let Some(max_clock_drift) = config.max_clock_drift {
    //     if Utc::now().signed_duration_since(meta.issued).num_minutes() > max_clock_drift {
    //         return Err(Error::SystemClockUnsynced);
    //     }
    // }

    if meta.ttl != 0 && Utc::now() > meta.expiry {
        return Err(Error::CerificateFileExpired);
    }
    Ok(())
}
