use lazy_static::lazy_static;
use std::env;

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

pub use errors::Error;

lazy_static! {
    pub static ref API_URL: String =
        env::var("KEYGEN_API_URL").unwrap_or("https://api.keygen.sh".to_string());
    pub static ref API_VERSION: String =
        env::var("KEYGEN_API_VERSION").unwrap_or("1.7".to_string());
    pub static ref API_PREFIX: String = env::var("KEYGEN_API_PREFIX").unwrap_or("v1".to_string());
    pub static ref ACCOUNT: String = env::var("KEYGEN_ACCOUNT").unwrap_or_default();
    pub static ref PRODUCT: String = env::var("KEYGEN_PRODUCT").unwrap_or_default();
    pub static ref PACKAGE: String = env::var("KEYGEN_PACKAGE").unwrap_or_default();
    pub static ref ENVIRONMENT: Option<String> = env::var("KEYGEN_ENVIRONMENT").ok();
    pub static ref LICENSE_KEY: Option<String> = env::var("KEYGEN_LICENSE_KEY").ok();
    pub static ref TOKEN: Option<String> = env::var("KEYGEN_TOKEN").ok();
    pub static ref PUBLIC_KEY: Option<String> = env::var("KEYGEN_PUBLIC_KEY").ok();
    pub static ref USER_AGENT: Option<String> = env::var("KEYGEN_USER_AGENT").ok();
}
