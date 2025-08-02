use keygen_rs::{
    config::{self, KeygenConfig},
    entitlement::{Entitlement, UpdateEntitlementRequest},
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

    let args: Vec<String> = env::args().collect();
    let entitlement_id = if args.len() > 1 {
        args[1].clone()
    } else {
        env::var("KEYGEN_ENTITLEMENT_ID").unwrap_or_else(|_| {
            println!("Usage: cargo run --example update_entitlement <entitlement_id>");
            std::process::exit(1);
        })
    };

    let entitlement = Entitlement::get(&entitlement_id).await?;

    let mut updated_metadata = HashMap::new();
    updated_metadata.insert("feature_level".to_string(), serde_json::Value::String("enterprise".to_string()));
    updated_metadata.insert("max_users".to_string(), serde_json::Value::Number(serde_json::Number::from(500)));

    let update_request = UpdateEntitlementRequest {
        name: Some("Enterprise Features (Updated)".to_string()),
        code: None,
        metadata: Some(updated_metadata),
    };

    match entitlement.update(update_request).await {
        Ok(updated_entitlement) => {
            println!("Entitlement updated: {}", updated_entitlement.code);
        }
        Err(e) => {
            println!("Failed to update entitlement: {:?}", e);
        }
    }

    Ok(())
}