use keygen_rs::{
    config::{self, KeygenConfig},
    license,
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // Suspend a license
    let license_id = env::var("LICENSE_ID").expect("LICENSE_ID must be set (get from list_licenses example)");
    
    match license::suspend(&license_id).await {
        Ok(license) => {
            println!("✅ License suspended successfully!");
            println!("ID: {}", license.id);
            println!("Key: {}", license.key);
            println!("Status: {:?}", license.status);
        },
        Err(e) => {
            println!("❌ Failed to suspend license: {:?}", e);
        }
    }

    Ok(())
}