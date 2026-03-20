use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct KeygenConfig {
    pub account: String,
    pub product: String,
    pub license_key: Option<String>,
    pub public_key: Option<String>,
    pub api_url: Option<String>,
    pub api_version: Option<String>,
    pub api_prefix: Option<String>,
    pub environment: Option<String>,
    pub user_agent: Option<String>,
    pub package: Option<String>,
    pub platform: Option<String>,
    pub max_clock_drift: Option<i64>,
    pub verify_keygen_signature: Option<bool>,
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

#[napi]
pub fn set_config(config: KeygenConfig) -> Result<()> {
    keygen_rs::config::set_config((&config).into()).map_err(to_napi_error)
}

#[napi]
pub fn get_config() -> Result<KeygenConfig> {
    keygen_rs::config::get_config()
        .map(KeygenConfig::from)
        .map_err(to_napi_error)
}

#[napi]
pub fn reset_config() -> Result<()> {
    keygen_rs::config::reset_config().map_err(to_napi_error)
}
