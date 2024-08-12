pub mod artifact;
pub mod certificate;
pub mod client;
pub mod component;
pub mod decryptor;
pub mod entitlement;
pub mod errors;
pub mod license;
pub mod license_file;
pub mod log;
pub mod machine;
pub mod machine_file;
pub mod process;
pub mod release;
pub mod upgrade;
pub mod validate;
pub mod verifier;
pub mod webhook;

use std::sync::RwLock;

pub use errors::Error;

use lazy_static::lazy_static;

#[derive(Clone, Debug)]
pub struct KeygenConfig {
    pub api_url: String,
    pub api_version: String,
    pub api_prefix: String,
    pub account: String,
    pub product: String,
    pub package: String,
    pub environment: Option<String>,
    pub license_key: Option<String>,
    pub token: Option<String>,
    pub public_key: Option<String>,
    pub user_agent: Option<String>,
}

impl Default for KeygenConfig {
    fn default() -> Self {
        KeygenConfig {
            api_url: "https://api.keygen.sh".to_string(),
            api_version: "1.7".to_string(),
            api_prefix: "v1".to_string(),
            account: String::new(),
            product: String::new(),
            package: String::new(),
            environment: None,
            license_key: None,
            token: None,
            public_key: None,
            user_agent: None,
        }
    }
}

lazy_static! {
    static ref KEYGEN_CONFIG: RwLock<KeygenConfig> = RwLock::new(KeygenConfig::default());
}

pub fn get_config() -> KeygenConfig {
    KEYGEN_CONFIG.read().unwrap().clone()
}

pub fn set_config(config: KeygenConfig) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    *current_config = config;
}

pub fn set_api_url(api_url: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.api_url = api_url.to_string();
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

pub fn set_product(product: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.product = product.to_string();
}

pub fn set_package(package: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.package = package.to_string();
}

pub fn set_environment(environment: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.environment = Some(environment.to_string());
}

pub fn set_license_key(license_key: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.license_key = Some(license_key.to_string());
}

pub fn set_token(token: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.token = Some(token.to_string());
}

pub fn set_public_key(public_key: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.public_key = Some(public_key.to_string());
}

pub fn set_user_agent(user_agent: &str) {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    current_config.user_agent = Some(user_agent.to_string());
}

pub fn reset_config() {
    let mut current_config = KEYGEN_CONFIG.write().unwrap();
    *current_config = KeygenConfig::default();
}
