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

    // List all licenses
    match license::list().await {
        Ok(licenses) => {
            println!("✅ Found {} licenses:", licenses.len());
            for license in licenses {
                println!("  ID: {}", license.id);
                println!("  Key: {}", license.key);
                println!("  Status: {:?}", license.status);
                if let Some(expiry) = license.expiry {
                    println!("  Expiry: {}", expiry);
                }
                println!("  Created: {}", license.created);
                println!("  ---");
            }
        },
        Err(e) => {
            println!("❌ Failed to list licenses: {:?}", e);
        }
    }

    Ok(())
}