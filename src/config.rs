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

pub fn set_config(config: KeygenConfig) -> Result<(), Error> {
    let mut current_config = KEYGEN_CONFIG
        .write()
        .map_err(|_| Error::UnexpectedError("Config lock poisoned".to_string()))?;
    *current_config = config;
    Ok(())
}

pub fn set_api_url(api_url: &str) -> Result<(), Error> {
    let mut current_config = KEYGEN_CONFIG
        .write()
        .map_err(|_| Error::UnexpectedError("Config lock poisoned".to_string()))?;
    current_config.api_url = api_url.to_string();
    Ok(())
}

pub fn set_api_version(api_version: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.api_version = api_version.to_string();
}

pub fn set_api_prefix(api_prefix: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.api_prefix = api_prefix.to_string();
}

pub fn set_account(account: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.account = account.to_string();
}

#[cfg(feature = "license-key")]
pub fn set_product(product: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.product = product.to_string();
}

#[cfg(feature = "license-key")]
pub fn set_package(package: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.package = package.to_string();
}

pub fn set_environment(environment: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.environment = Some(environment.to_string());
}

#[cfg(feature = "license-key")]
pub fn set_license_key(license_key: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.license_key = Some(license_key.to_string());
}

#[cfg(feature = "token")]
pub fn set_token(token: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.token = Some(token.to_string());
}

#[cfg(feature = "license-key")]
pub fn set_public_key(public_key: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.public_key = Some(public_key.to_string());
}

#[cfg(feature = "license-key")]
pub fn set_platform(platform: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.platform = Some(platform.to_string());
}

pub fn set_user_agent(user_agent: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.user_agent = Some(user_agent.to_string());
}

#[cfg(feature = "license-key")]
pub fn set_max_clock_drift(max_clock_drift: i64) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.max_clock_drift = Some(max_clock_drift);
}

pub fn reset_config() {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    *current_config = KeygenConfig::default();
}
