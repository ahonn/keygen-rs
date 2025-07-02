use keygen_rs::{
    config::{self, KeygenConfig},
    license,
    errors::Error,
};
use std::env;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // Create metadata for the license
    let mut metadata = HashMap::new();
    metadata.insert("customer_name".to_string(), serde_json::Value::String("John Doe".to_string()));
    metadata.insert("license_type".to_string(), serde_json::Value::String("premium".to_string()));

    // Create a new license
    let policy_id = env::var("POLICY_ID").expect("POLICY_ID must be set (get from list_policies example)");
    
    match license::create(&policy_id, None, Some(metadata)).await {
        Ok(license) => {
            println!("✅ License created successfully!");
            println!("ID: {}", license.id);
            println!("Key: {}", license.key);
            println!("Status: {:?}", license.status);
            if let Some(expiry) = license.expiry {
                println!("Expiry: {}", expiry);
            }
            if let Some(metadata) = license.metadata {
                println!("Metadata: {:?}", metadata);
            }
        },
        Err(e) => {
            println!("❌ Failed to create license: {:?}", e);
        }
    }

    Ok(())
}