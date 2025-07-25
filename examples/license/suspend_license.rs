use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::License,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    }).expect("Failed to set config");

    // Get the license first, then suspend it
    let license_id =
        env::var("LICENSE_ID").expect("LICENSE_ID must be set (get from list_licenses example)");

    // First get the license
    match License::get(&license_id).await {
        Ok(license) => {
            println!("📄 Found license: {}", license.key);

            // Then suspend it
            match license.suspend().await {
                Ok(suspended_license) => {
                    println!("✅ License suspended successfully!");
                    println!("ID: {}", suspended_license.id);
                    println!("Key: {}", suspended_license.key);
                    println!("Status: {:?}", suspended_license.status);
                    println!("Uses: {:?}", suspended_license.uses);
                    println!("Max Machines: {:?}", suspended_license.max_machines);
                    println!("Max Cores: {:?}", suspended_license.max_cores);
                    println!("Max Uses: {:?}", suspended_license.max_uses);
                    println!("Max Processes: {:?}", suspended_license.max_processes);
                    println!("Protected: {:?}", suspended_license.protected);
                    println!("Suspended: {:?}", suspended_license.suspended);
                }
                Err(e) => {
                    println!("❌ Failed to suspend license: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get license: {:?}", e);
        }
    }

    Ok(())
}
