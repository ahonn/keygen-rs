use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct ServiceInfo {
    pub timestamp: Option<String>,
    pub api_version: Option<String>,
    pub message: Option<String>,
    pub headers: serde_json::Value,
}

#[napi(object)]
#[derive(Clone)]
pub struct PingResponse {
    pub message: String,
    pub version: Option<String>,
    pub timestamp: Option<String>,
}

#[napi]
pub async fn get_service_info() -> Result<ServiceInfo> {
    keygen_rs::service::get_service_info()
        .await
        .map(|info| ServiceInfo {
            timestamp: info.timestamp,
            api_version: info.api_version,
            message: info.message,
            headers: serde_json::to_value(info.headers).unwrap_or_default(),
        })
        .map_err(to_napi_error)
}

#[napi]
pub async fn ping() -> Result<PingResponse> {
    keygen_rs::service::ping()
        .await
        .map(|resp| PingResponse {
            message: resp.message,
            version: resp.version,
            timestamp: resp.timestamp,
        })
        .map_err(to_napi_error)
}

#[napi]
pub async fn supports_product_code() -> Result<bool> {
    keygen_rs::service::supports_product_code()
        .await
        .map_err(to_napi_error)
}
