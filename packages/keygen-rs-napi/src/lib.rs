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

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str, label: &str) -> napi::Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| napi::Error::new(napi::Status::InvalidArg, format!("Invalid {label}: {e}")))
}

fn enum_to_string<T: serde::Serialize>(val: &T) -> Option<String> {
    serde_json::to_value(val)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
}

fn to_metadata(
    v: serde_json::Value,
) -> napi::Result<std::collections::HashMap<String, serde_json::Value>> {
    serde_json::from_value(v)
        .map_err(|e| napi::Error::new(napi::Status::InvalidArg, format!("Invalid metadata: {e}")))
}

fn opt_metadata(
    v: Option<serde_json::Value>,
) -> napi::Result<Option<std::collections::HashMap<String, serde_json::Value>>> {
    v.map(to_metadata).transpose()
}
