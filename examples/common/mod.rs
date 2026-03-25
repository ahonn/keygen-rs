#![allow(dead_code)]

use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn load_env() {
    dotenv::dotenv().ok();
}

pub fn configure_admin() -> Result<(), Error> {
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: required_env("KEYGEN_ACCOUNT"),
        token: Some(required_env("KEYGEN_ADMIN_TOKEN")),
        ..KeygenConfig::default()
    })
}

pub fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("{name} must be set"))
}

pub fn product_id_from_env() -> String {
    env::var("KEYGEN_PRODUCT_ID")
        .or_else(|_| env::var("KEYGEN_PRODUCT"))
        .expect("KEYGEN_PRODUCT_ID or KEYGEN_PRODUCT must be set")
}

pub fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after unix epoch")
        .as_millis()
        .to_string()
}
