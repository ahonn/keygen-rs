use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::license::License;
use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseFileDataset {
    pub license: License,
    pub issued: String,
    pub expiry: String,
    pub ttl: i32,
}

#[wasm_bindgen(js_name = "licenseFileFromCert")]
pub fn license_file_from_cert(key: String, content: String) -> Result<JsValue, JsError> {
    let lf = keygen_rs::license_file::LicenseFile::from_cert(&key, &content)
        .map(LicenseFile::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&lf).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "verifyLicenseFile")]
pub fn verify_license_file(certificate: String) -> Result<(), JsError> {
    let lf = keygen_rs::license_file::LicenseFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    lf.verify().map_err(to_js_error)
}

#[wasm_bindgen(js_name = "decryptLicenseFile")]
pub fn decrypt_license_file(certificate: String, key: String) -> Result<JsValue, JsError> {
    let lf = keygen_rs::license_file::LicenseFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    let ds = lf.decrypt(&key).map_err(to_js_error)?;
    let dataset = LicenseFileDataset {
        license: License::from(ds.license),
        issued: ds.issued.to_rfc3339(),
        expiry: ds.expiry.to_rfc3339(),
        ttl: ds.ttl,
    };
    serde_wasm_bindgen::to_value(&dataset).map_err(|e| JsError::new(&e.to_string()))
}
