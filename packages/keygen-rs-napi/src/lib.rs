#![deny(clippy::all)]

use keygen_rs::errors::ErrorMeta;

pub mod config;
pub mod license;
pub mod license_file;
pub mod machine;
pub mod machine_file;
pub mod service;

pub mod arch;
pub mod artifact;
pub mod channel;
pub mod component;
pub mod entitlement;
pub mod environment;
pub mod group;
pub mod keygen_package;
pub mod keygen_platform;
pub mod policy;
pub mod product;
pub mod release;
pub mod token_module;
pub mod user;
pub mod webhook;

fn to_napi_error(e: keygen_rs::errors::Error) -> napi::Error {
    let code = e.code();
    let detail = e.detail();
    napi::Error::new(napi::Status::GenericFailure, format!("[{code}] {detail}"))
}
