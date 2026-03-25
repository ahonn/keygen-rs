#![deny(clippy::all)]

use keygen_rs::errors::ErrorMeta;
use wasm_bindgen::JsError;

pub mod config;
pub mod license;
pub mod license_file;
pub mod machine;
pub mod machine_file;
pub mod service;

pub mod component;
pub mod entitlement;
pub mod environment;
pub mod group;
pub mod policy;
pub mod product;
pub mod release;
pub mod token_module;
pub mod user;
pub mod webhook;

fn to_js_error(e: keygen_rs::errors::Error) -> JsError {
    let code = e.code();
    let detail = e.detail();
    JsError::new(&format!("[{code}] {detail}"))
}

fn to_metadata(
    v: serde_json::Value,
) -> Result<std::collections::HashMap<String, serde_json::Value>, JsError> {
    serde_json::from_value(v).map_err(|e| JsError::new(&format!("Invalid metadata: {e}")))
}

fn opt_metadata(
    v: Option<serde_json::Value>,
) -> Result<Option<std::collections::HashMap<String, serde_json::Value>>, JsError> {
    v.map(to_metadata).transpose()
}

fn parse_enum<T: serde::de::DeserializeOwned>(s: &str, label: &str) -> Result<T, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid {label}: {e}")))
}
