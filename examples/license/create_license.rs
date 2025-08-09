use chrono::{DateTime, Utc};
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::{License, LicenseCreateRequest},
};
use std::collections::HashMap;
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
    })
    .expect("Failed to set config");

    // Get required policy ID
    let policy_id = env::var("KEYGEN_POLICY_ID")
        .expect("KEYGEN_POLICY_ID must be set (get from list_policies example)");

    // Create metadata for the license
    let mut metadata = HashMap::new();
    metadata.insert(
        "email".to_string(),
        serde_json::Value::String("example@gmail.com".to_string()),
    );
    metadata.insert(
        "provider".to_string(),
        serde_json::Value::String("stripe".to_string()),
    );

    // Set expiry to 2 years from now
    let expiry: DateTime<Utc> = Utc::now() + chrono::Duration::days(730);

    // Optional: Get user ID and group ID from environment if available
    let owner_id = env::var("KEYGEN_USER_ID").ok();
    let group_id = env::var("KEYGEN_GROUP_ID").ok();

    let mut request = LicenseCreateRequest::new(policy_id)
        .with_name("Standard License".to_string())
        .with_expiry(expiry)
        .with_max_machines(3)
        .with_metadata(metadata);

    if let Some(uid) = owner_id {
        request = request.with_owner_id(uid);
    }
    if let Some(gid) = group_id {
        request = request.with_group_id(gid);
    }

    match License::create(request).await {
        Ok(license) => {
            println!("License created: {} ({})", license.id, license.key);
        }
        Err(e) => {
            println!("Failed to create license: {e:?}");
        }
    }

    Ok(())
}
