use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::license::License;
use crate::machine::Machine;
use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineFile {
    pub id: String,
    pub certificate: String,
    pub issued: String,
    pub expiry: String,
    pub ttl: i32,
}

impl From<keygen_rs::machine_file::MachineFile> for MachineFile {
    fn from(mf: keygen_rs::machine_file::MachineFile) -> Self {
        MachineFile {
            id: mf.id,
            certificate: mf.certificate,
            issued: mf.issued.to_rfc3339(),
            expiry: mf.expiry.to_rfc3339(),
            ttl: mf.ttl,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MachineFileDataset {
    pub license: License,
    pub machine: Machine,
    pub issued: String,
    pub expiry: String,
    pub ttl: i32,
}

#[wasm_bindgen(js_name = "machineFileFromCert")]
pub fn machine_file_from_cert(key: String, content: String) -> Result<JsValue, JsError> {
    let mf = keygen_rs::machine_file::MachineFile::from_cert(&key, &content)
        .map(MachineFile::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&mf).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "verifyMachineFile")]
pub fn verify_machine_file(certificate: String) -> Result<(), JsError> {
    let mf = keygen_rs::machine_file::MachineFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    mf.verify().map_err(to_js_error)
}

#[wasm_bindgen(js_name = "decryptMachineFile")]
pub fn decrypt_machine_file(certificate: String, key: String) -> Result<JsValue, JsError> {
    let mf = keygen_rs::machine_file::MachineFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    let ds = mf.decrypt(&key).map_err(to_js_error)?;
    let dataset = MachineFileDataset {
        license: License::from(ds.license),
        machine: Machine::from(ds.machine),
        issued: ds.issued.to_rfc3339(),
        expiry: ds.expiry.to_rfc3339(),
        ttl: ds.ttl,
    };
    serde_wasm_bindgen::to_value(&dataset).map_err(|e| JsError::new(&e.to_string()))
}
