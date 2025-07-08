use keygen_rs::{
    config::{self, KeygenConfig},
    license::License,
    errors::Error,
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
    });

    // Get the license first, then reinstate it
    let license_id = env::var("LICENSE_ID").expect("LICENSE_ID must be set (get from list_licenses example)");
    
    // First get the license
    match License::get(&license_id).await {
        Ok(license) => {
            println!("📄 Found license: {}", license.key);
            
            // Then reinstate it
            match license.reinstate().await {
                Ok(reinstated_license) => {
                    println!("✅ License reinstated successfully!");
                    println!("ID: {}", reinstated_license.id);
                    println!("Key: {}", reinstated_license.key);
                    println!("Status: {:?}", reinstated_license.status);
                    println!("Uses: {:?}", reinstated_license.uses);
                    println!("Max Machines: {:?}", reinstated_license.max_machines);
                    println!("Max Cores: {:?}", reinstated_license.max_cores);
                    println!("Max Uses: {:?}", reinstated_license.max_uses);
                    println!("Max Processes: {:?}", reinstated_license.max_processes);
                    println!("Protected: {:?}", reinstated_license.protected);
                    println!("Suspended: {:?}", reinstated_license.suspended);
                },
                Err(e) => {
                    println!("❌ Failed to reinstate license: {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to get license: {:?}", e);
        }
    }

    Ok(())
}