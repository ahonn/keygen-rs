use chrono::Utc;
use keygen_rs::{
    config::{self, KeygenConfig},
    entitlement::{CreateEntitlementRequest, Entitlement},
    errors::Error,
};
use std::{collections::HashMap, env};

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

    // Create metadata for the entitlement
    let mut metadata = HashMap::new();
    metadata.insert(
        "feature_level".to_string(),
        serde_json::Value::String("premium".to_string()),
    );
    metadata.insert(
        "max_users".to_string(),
        serde_json::Value::Number(serde_json::Number::from(100)),
    );

    // Create a new entitlement
    let timestamp = Utc::now().timestamp();
    let request = CreateEntitlementRequest {
        name: Some("Premium Features".to_string()),
        code: format!("premium-features-{timestamp}"),
        metadata: Some(metadata),
    };

    match Entitlement::create(request).await {
        Ok(entitlement) => {
            println!("Entitlement created: {}", entitlement.code);
        }
        Err(e) => {
            println!("Failed to create entitlement: {e:?}");
        }
    }

    Ok(())
}
