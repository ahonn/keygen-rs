use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeygenConfig {
    pub account: String,
    pub product: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_clock_drift: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_keygen_signature: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl From<&KeygenConfig> for keygen_rs::config::KeygenConfig {
    fn from(cfg: &KeygenConfig) -> Self {
        keygen_rs::config::KeygenConfig {
            account: cfg.account.clone(),
            api_url: cfg
                .api_url
                .clone()
                .unwrap_or_else(|| "https://api.keygen.sh".to_string()),
            api_version: cfg.api_version.clone().unwrap_or_else(|| "1.7".to_string()),
            api_prefix: cfg.api_prefix.clone().unwrap_or_else(|| "v1".to_string()),
            environment: cfg.environment.clone(),
            user_agent: cfg.user_agent.clone(),
            product: cfg.product.clone(),
            package: cfg.package.clone().unwrap_or_default(),
            license_key: cfg.license_key.clone(),
            public_key: cfg.public_key.clone(),
            platform: cfg.platform.clone(),
            max_clock_drift: cfg.max_clock_drift.or(Some(5)),
            verify_keygen_signature: cfg.verify_keygen_signature.or(Some(true)),
            token: cfg.token.clone(),
        }
    }
}

impl From<keygen_rs::config::KeygenConfig> for KeygenConfig {
    fn from(cfg: keygen_rs::config::KeygenConfig) -> Self {
        KeygenConfig {
            account: cfg.account,
            product: cfg.product,
            license_key: cfg.license_key,
            public_key: cfg.public_key,
            api_url: Some(cfg.api_url),
            api_version: Some(cfg.api_version),
            api_prefix: Some(cfg.api_prefix),
            environment: cfg.environment,
            user_agent: cfg.user_agent,
            package: Some(cfg.package),
            platform: cfg.platform,
            max_clock_drift: cfg.max_clock_drift,
            verify_keygen_signature: cfg.verify_keygen_signature,
            token: cfg.token,
        }
    }
}

#[wasm_bindgen(js_name = "setConfig")]
pub fn set_config(config: JsValue) -> Result<(), JsError> {
    let cfg: KeygenConfig =
        serde_wasm_bindgen::from_value(config).map_err(|e| JsError::new(&e.to_string()))?;
    keygen_rs::config::set_config((&cfg).into()).map_err(to_js_error)
}

#[wasm_bindgen(js_name = "getConfig")]
pub fn get_config() -> Result<JsValue, JsError> {
    let cfg = keygen_rs::config::get_config().map_err(to_js_error)?;
    let wasm_cfg = KeygenConfig::from(cfg);
    serde_wasm_bindgen::to_value(&wasm_cfg).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "resetConfig")]
pub fn reset_config() -> Result<(), JsError> {
    keygen_rs::config::reset_config().map_err(to_js_error)
}
