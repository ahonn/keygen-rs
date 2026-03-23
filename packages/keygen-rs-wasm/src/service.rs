use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub timestamp: Option<String>,
    pub api_version: Option<String>,
    pub message: Option<String>,
    pub headers: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub message: String,
    pub version: Option<String>,
    pub timestamp: Option<String>,
}

#[wasm_bindgen(js_name = "getServiceInfo")]
pub async fn get_service_info() -> Result<JsValue, JsError> {
    let info = keygen_rs::service::get_service_info()
        .await
        .map(|info| ServiceInfo {
            timestamp: info.timestamp,
            api_version: info.api_version,
            message: info.message,
            headers: serde_json::to_value(info.headers).unwrap_or_default(),
        })
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&info).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "ping")]
pub async fn ping() -> Result<JsValue, JsError> {
    let resp = keygen_rs::service::ping()
        .await
        .map(|resp| PingResponse {
            message: resp.message,
            version: resp.version,
            timestamp: resp.timestamp,
        })
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&resp).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "supportsProductCode")]
pub async fn supports_product_code() -> Result<bool, JsError> {
    keygen_rs::service::supports_product_code()
        .await
        .map_err(to_js_error)
}
