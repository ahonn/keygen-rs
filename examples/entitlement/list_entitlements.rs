use keygen_rs::{
    config::{self, KeygenConfig},
    entitlement::Entitlement,
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
    })?;

    match Entitlement::list(None).await {
        Ok(entitlements) => {
            println!("Found {} entitlements:", entitlements.len());
            for entitlement in entitlements {
                println!(
                    "  Code: {} | ID: {} | Name: {}",
                    entitlement.code,
                    entitlement.id,
                    entitlement.name.unwrap_or_else(|| "No name".to_string())
                );
            }
        }
        Err(e) => {
            println!("Failed to list entitlements: {:?}", e);
        }
    }

    Ok(())
}
