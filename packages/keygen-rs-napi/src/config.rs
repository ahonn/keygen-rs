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

impl TryFrom<&KeygenConfig> for keygen_rs::config::KeygenConfig {
    type Error = keygen_rs::api_version::ApiVersionParseError;

    fn try_from(cfg: &KeygenConfig) -> std::result::Result<Self, Self::Error> {
        let mut config = keygen_rs::config::KeygenConfig::default();
        config.account.clone_from(&cfg.account);
        config.product.clone_from(&cfg.product);
        config.license_key.clone_from(&cfg.license_key);
        config.public_key.clone_from(&cfg.public_key);
        config.environment.clone_from(&cfg.environment);
        config.user_agent.clone_from(&cfg.user_agent);
        config.platform.clone_from(&cfg.platform);
        config.token.clone_from(&cfg.token);
        config.api_url = cfg
            .api_url
            .clone()
            .unwrap_or_else(|| "https://api.keygen.sh".to_string());
        config.api_version = cfg.api_version.as_deref().unwrap_or("1.8").parse()?;
        config.api_prefix = cfg.api_prefix.clone().unwrap_or_else(|| "v1".to_string());
        config.package = cfg.package.clone().unwrap_or_default();
        config.max_clock_drift = cfg.max_clock_drift.or(Some(5));
        config.verify_keygen_signature = cfg.verify_keygen_signature.or(Some(true));
        Ok(config)
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
            api_version: Some(cfg.api_version.to_string()),
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
    let config = keygen_rs::config::KeygenConfig::try_from(&config)
        .map_err(|error| napi::Error::new(Status::InvalidArg, error.to_string()))?;
    keygen_rs::config::set_config(config).map_err(to_napi_error)
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
