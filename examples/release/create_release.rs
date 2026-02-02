use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    release::{CreateReleaseRequest, Release, ReleaseChannel, ReleaseStatus},
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

    // Get product ID from environment
    let product_id = env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set");

    // Create a new release
    let version = format!("1.0.{}", chrono::Utc::now().timestamp() % 1000);
    let mut metadata = HashMap::new();
    metadata.insert("sha256".to_string(), serde_json::json!("abc123def456..."));
    metadata.insert(
        "releaseNotes".to_string(),
        serde_json::json!("Bug fixes and performance improvements"),
    );

    let request = CreateReleaseRequest {
        version: version.clone(),
        channel: ReleaseChannel::Stable,
        product_id,
        name: Some(format!("v{}", version)),
        description: Some("Initial release with bug fixes and improvements".to_string()),
        status: Some(ReleaseStatus::Draft),
        tag: Some(format!("v{}", version)),
        metadata: Some(metadata),
    };

    match Release::create(request).await {
        Ok(release) => {
            println!("Release created successfully!");
            println!("  ID: {}", release.id);
            println!("  Version: {}", release.version);
            println!("  Channel: {:?}", release.channel);
            println!("  Status: {:?}", release.status);
            println!("  Name: {:?}", release.name);
            println!("  Tag: {:?}", release.tag);
            println!("  Created: {}", release.created);
            println!("  Product ID: {:?}", release.product_id);
        }
        Err(e) => {
            println!("Failed to create release: {e:?}");
        }
    }

    Ok(())
}
