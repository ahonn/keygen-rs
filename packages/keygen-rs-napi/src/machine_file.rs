use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::license::License;
use crate::machine::Machine;
use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct MachineFileDataset {
    pub license: License,
    pub machine: Machine,
    pub issued: String,
    pub expiry: String,
    pub ttl: i32,
}

#[napi]
pub fn machine_file_from_cert(key: String, content: String) -> Result<MachineFile> {
    keygen_rs::machine_file::MachineFile::from_cert(&key, &content)
        .map(MachineFile::from)
        .map_err(to_napi_error)
}

#[napi]
pub fn verify_machine_file(certificate: String) -> Result<()> {
    let mf = keygen_rs::machine_file::MachineFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    mf.verify().map_err(to_napi_error)
}

#[napi]
pub fn decrypt_machine_file(certificate: String, key: String) -> Result<MachineFileDataset> {
    let mf = keygen_rs::machine_file::MachineFile {
        id: String::new(),
        certificate,
        issued: chrono::Utc::now(),
        expiry: chrono::Utc::now(),
        ttl: 0,
    };
    mf.decrypt(&key)
        .map(|ds| MachineFileDataset {
            license: License::from(ds.license),
            machine: Machine::from(ds.machine),
            issued: ds.issued.to_rfc3339(),
            expiry: ds.expiry.to_rfc3339(),
            ttl: ds.ttl,
        })
        .map_err(to_napi_error)
}
