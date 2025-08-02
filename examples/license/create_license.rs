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
    }).expect("Failed to set config");

    // Get required policy ID
    let policy_id =
        env::var("KEYGEN_POLICY_ID").expect("KEYGEN_POLICY_ID must be set (get from list_policies example)");

    // Example 1: Create a basic license with all optional parameters

    // Create metadata for the license
    let mut metadata = HashMap::new();
    metadata.insert(
        "customer_name".to_string(),
        serde_json::Value::String("John Doe".to_string()),
    );
    metadata.insert(
        "license_type".to_string(),
        serde_json::Value::String("premium".to_string()),
    );
    metadata.insert(
        "features".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("feature_a".to_string()),
            serde_json::Value::String("feature_b".to_string()),
        ]),
    );

    // Set expiry to 1 year from now
    let expiry: DateTime<Utc> = Utc::now() + chrono::Duration::days(365);

    // Optional: Get user ID and group ID from environment if available
    let owner_id = env::var("KEYGEN_USER_ID").ok();
    let group_id = env::var("KEYGEN_GROUP_ID").ok();

    let mut request = LicenseCreateRequest::new(policy_id.clone())
        .with_name("Premium License for John Doe".to_string())
        .with_expiry(expiry)
        .with_max_machines(1)
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
            println!("Failed to create license: {:?}", e);
            return Err(e);
        }
    }

    // Example 2: Create a minimal license (only policy required)

    let minimal_request = LicenseCreateRequest::new(policy_id.clone());

    match License::create(minimal_request).await {
        Ok(license) => {
            println!("License created: {} ({})", license.id, license.key);
        }
        Err(e) => {
            println!("Failed to create license: {:?}", e);
        }
    }

    // Example 3: Create a license with just name and machine limit

    let standard_request = LicenseCreateRequest::new(policy_id.clone())
        .with_name("Standard License".to_string())
        .with_max_machines(3);

    match License::create(standard_request).await {
        Ok(license) => {
            println!("License created: {} ({})", license.id, license.key);
        }
        Err(e) => {
            println!("Failed to create license: {:?}", e);
        }
    }

    // Example 4: Create a comprehensive license with all available parameters

    let mut comprehensive_metadata = HashMap::new();
    comprehensive_metadata.insert(
        "tier".to_string(),
        serde_json::Value::String("enterprise".to_string()),
    );
    comprehensive_metadata.insert(
        "support_level".to_string(),
        serde_json::Value::String("premium".to_string()),
    );
    comprehensive_metadata.insert(
        "features".to_string(),
        serde_json::Value::Array(vec![
            serde_json::Value::String("advanced_analytics".to_string()),
            serde_json::Value::String("priority_support".to_string()),
            serde_json::Value::String("custom_integrations".to_string()),
        ]),
    );

    // Set expiry to 2 years from now
    let long_expiry: DateTime<Utc> = Utc::now() + chrono::Duration::days(730);

    let comprehensive_request = LicenseCreateRequest::new(policy_id)
        .with_name("Enterprise License - Full Features".to_string())
        .with_key("ENTERPRISE-2024-FULL-ACCESS".to_string())
        .with_expiry(long_expiry)
        .with_max_machines(50)
        .with_max_processes(100)
        .with_max_users(25)
        .with_max_cores(200)
        .with_max_uses(10000)
        .with_protected(true)
        .with_suspended(false)
        .with_permissions(vec![
            "activate".to_string(),
            "deactivate".to_string(),
            "read".to_string(),
            "update".to_string(),
            "manage".to_string(),
        ])
        .with_metadata(comprehensive_metadata);

    match License::create(comprehensive_request).await {
        Ok(license) => {
            println!("License created: {} ({})", license.id, license.key);
        }
        Err(e) => {
            println!("Failed to create license: {:?}", e);
        }
    }
    Ok(())
}
