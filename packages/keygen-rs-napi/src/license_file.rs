use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::license::License;
use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct LicenseFile {
    pub id: String,
    pub certificate: String,
    pub issued: String,
    pub expiry: String,
    pub ttl: i32,
}

impl From<keygen_rs::license_file::LicenseFile> for LicenseFile {
    fn from(lf: keygen_rs::license_file::LicenseFile) -> Self {
        LicenseFile {
            id: lf.id,
            certificate: lf.certificate,
            issued: lf.issued.to_rfc3339(),
            expiry: lf.expiry.to_rfc3339(),
            ttl: lf.ttl,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct LicenseFileDataset {
    pub license: License,
    pub issued: String,
    pub expiry: String,
    pub ttl: i32,
}

#[napi]
pub fn license_file_from_cert(key: String, content: String) -> Result<LicenseFile> {
    keygen_rs::license_file::LicenseFile::from_cert(&key, &content)
        .map(LicenseFile::from)
        .map_err(to_napi_error)
}

#[napi]
pub fn verify_license_file(certificate: String) -> Result<()> {
    let lf = keygen_rs::license_file::LicenseFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    lf.verify().map_err(to_napi_error)
}

#[napi]
pub fn decrypt_license_file(certificate: String, key: String) -> Result<LicenseFileDataset> {
    let lf = keygen_rs::license_file::LicenseFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    lf.decrypt(&key)
        .map(|ds| LicenseFileDataset {
            license: License::from(ds.license),
            issued: ds.issued.to_rfc3339(),
            expiry: ds.expiry.to_rfc3339(),
            ttl: ds.ttl,
        })
        .map_err(to_napi_error)
}
