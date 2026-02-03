//! Global configuration management for keygen-rs.
//!
//! This module provides thread-safe global configuration for the Keygen API client.
//! Configuration can be set once and accessed throughout the application.
//!
//! # Example
//! ```no_run
//! use keygen_rs::config::{set_config, KeygenConfig};
//!
//! set_config(KeygenConfig {
//!     account: "your-account-id".to_string(),
//!     product: "your-product-id".to_string(),
//!     license_key: Some("your-license-key".to_string()),
//!     public_key: Some("your-public-key".to_string()),
//!     ..Default::default()
//! }).expect("Failed to set config");
//! ```

use crate::errors::Error;
use lazy_static::lazy_static;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct KeygenConfig {
    // Common configuration
    pub api_url: String,
    pub api_version: String,
    pub api_prefix: String,
    pub account: String,
    pub environment: Option<String>,
    pub user_agent: Option<String>,

    // License Key Authentication configuration
    #[cfg(feature = "license-key")]
    pub product: String,
    #[cfg(feature = "license-key")]
    pub package: String,
    #[cfg(feature = "license-key")]
    pub license_key: Option<String>,
    #[cfg(feature = "license-key")]
    pub public_key: Option<String>,
    #[cfg(feature = "license-key")]
    pub platform: Option<String>,
    #[cfg(feature = "license-key")]
    pub max_clock_drift: Option<i64>,
    #[cfg(feature = "license-key")]
    pub verify_keygen_signature: Option<bool>,

    // Token Authentication configuration
    #[cfg(feature = "token")]
    pub token: Option<String>,
}

impl Default for KeygenConfig {
    fn default() -> Self {
        KeygenConfig {
            // Common defaults
            api_url: "https://api.keygen.sh".to_string(),
            api_version: "1.7".to_string(),
            api_prefix: "v1".to_string(),
            account: String::new(),
            environment: None,
            user_agent: None,

            // License Key Authentication defaults
            #[cfg(feature = "license-key")]
            product: String::new(),
            #[cfg(feature = "license-key")]
            package: String::new(),
            #[cfg(feature = "license-key")]
            license_key: None,
            #[cfg(feature = "license-key")]
            public_key: None,
            #[cfg(feature = "license-key")]
            platform: None,
            #[cfg(feature = "license-key")]
            max_clock_drift: Some(5),
            #[cfg(feature = "license-key")]
            verify_keygen_signature: Some(true),

            // Token Authentication defaults
            #[cfg(feature = "token")]
            token: None,
        }
    }
}

impl KeygenConfig {
    /// Create a license key authentication configuration
    #[cfg(feature = "license-key")]
    pub fn license_key(
        account: String,
        product: String,
        license_key: String,
        public_key: String,
    ) -> Self {
        KeygenConfig {
            account,
            product,
            license_key: Some(license_key),
            public_key: Some(public_key),
            ..Default::default()
        }
    }

    /// Create a token authentication configuration
    #[cfg(feature = "token")]
    pub fn token(account: String, token: String) -> Self {
        KeygenConfig {
            account,
            token: Some(token),
            ..Default::default()
        }
    }
}

lazy_static! {
    static ref KEYGEN_CONFIG: RwLock<KeygenConfig> = RwLock::new(KeygenConfig::default());
}

pub fn get_config() -> Result<KeygenConfig, Error> {
    KEYGEN_CONFIG
        .read()
        .map_err(|_| Error::UnexpectedError("Config lock poisoned".to_string()))?
        .clone()
        .into()
}

impl From<KeygenConfig> for Result<KeygenConfig, Error> {
    fn from(config: KeygenConfig) -> Self {
        Ok(config)
    }
}

fn update_config<F>(f: F) -> Result<(), Error>
where
    F: FnOnce(&mut KeygenConfig),
{
    let mut current_config = KEYGEN_CONFIG
        .write()
        .map_err(|_| Error::UnexpectedError("Config lock poisoned".to_string()))?;
    f(&mut current_config);
    Ok(())
}

pub fn set_config(config: KeygenConfig) -> Result<(), Error> {
    update_config(|current| *current = config)
}

pub fn set_api_url(api_url: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.api_url = api_url.to_string())
}

pub fn set_api_version(api_version: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.api_version = api_version.to_string())
}

pub fn set_api_prefix(api_prefix: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.api_prefix = api_prefix.to_string())
}

pub fn set_account(account: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.account = account.to_string())
}

#[cfg(feature = "license-key")]
pub fn set_product(product: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.product = product.to_string())
}

#[cfg(feature = "license-key")]
pub fn set_package(package: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.package = package.to_string())
}

pub fn set_environment(environment: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.environment = Some(environment.to_string()))
}

#[cfg(feature = "license-key")]
pub fn set_license_key(license_key: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.license_key = Some(license_key.to_string()))
}

#[cfg(feature = "token")]
pub fn set_token(token: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.token = Some(token.to_string()))
}

#[cfg(feature = "license-key")]
pub fn set_public_key(public_key: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.public_key = Some(public_key.to_string()))
}

#[cfg(feature = "license-key")]
pub fn set_platform(platform: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.platform = Some(platform.to_string()))
}

pub fn set_user_agent(user_agent: &str) -> Result<(), Error> {
    update_config(|cfg| cfg.user_agent = Some(user_agent.to_string()))
}

#[cfg(feature = "license-key")]
pub fn set_max_clock_drift(max_clock_drift: i64) -> Result<(), Error> {
    update_config(|cfg| cfg.max_clock_drift = Some(max_clock_drift))
}

pub fn reset_config() -> Result<(), Error> {
    update_config(|cfg| *cfg = KeygenConfig::default())
}
