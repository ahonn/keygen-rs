use chrono::{Duration, Utc};
use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::{License, LicenseUpdateRequest},
};
use std::{collections::HashMap, env};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    // Configure with Admin Token for management operations
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })
    .expect("Failed to set config");

    // Get license ID from environment or command line
    let license_id = env::var("KEYGEN_LICENSE_ID").expect("KEYGEN_LICENSE_ID must be set");

    // Fetch the license
    let license = License::get(&license_id).await?;
    println!("Current License Details:");
    println!("  ID: {}", license.id);
    println!("  Name: {:?}", license.name);
    println!("  Expiry: {:?}", license.expiry);
    println!("  Status: {:?}", license.status);
    println!("  Metadata: {:?}", license.metadata);

    // Update the license with new details
    let new_expiry = Utc::now() + Duration::days(365); // Extend by 1 year
    let mut metadata = HashMap::new();
    metadata.insert("updated_at".to_string(), serde_json::json!(Utc::now()));
    metadata.insert("support_tier".to_string(), serde_json::json!("premium"));
    metadata.insert("max_api_calls".to_string(), serde_json::json!(10000));

    let request = LicenseUpdateRequest::new()
        .with_name("Premium License - Updated".to_string())
        .with_expiry(new_expiry)
        .with_metadata(metadata);

    let updated_license = license.update(request).await?;

    println!("License updated: {}", updated_license.id);
    println!("Updated License Details:");
    println!("  ID: {}", updated_license.id);
    println!("  Name: {:?}", updated_license.name);
    println!("  Expiry: {:?}", updated_license.expiry);
    println!("  Status: {:?}", updated_license.status);
    println!("  Metadata: {:?}", updated_license.metadata);

    Ok(())
}
