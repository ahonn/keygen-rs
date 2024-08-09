use std::sync::Arc;
use std::env;
use lazy_static::lazy_static;

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
    pub static ref PUBLIC_KEY: String = env::var("KEYGEN_PUBLIC_KEY").unwrap_or_default();
}

#[derive(Clone)]
pub struct Keygen {
    config: Arc<Config>,
    client: Arc<client::Client>,
}

pub struct Config {
    pub account: String,
    pub product: String,
    pub environment: Option<String>,
    pub license_key: Option<String>,
    pub token: Option<String>,
    pub public_key: Option<String>,
    pub user_agent: Option<String>,
}

impl Keygen {
    pub fn new(config: Config) -> Self {
        unimplemented!()
    }

    pub async fn validate(&self, fingerprints: &[String]) -> Result<license::License, Error> {
        unimplemented!()
    }

    pub async fn upgrade(&self, options: upgrade::Options) -> Result<release::Release, Error> {
        unimplemented!()
    }
}
