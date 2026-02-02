use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    release::{CreateReleaseRequest, Release, ReleaseChannel, ReleaseStatus, UpdateReleaseRequest},
};
use std::{collections::HashMap, env};

/// This example demonstrates the full lifecycle of a release:
/// 1. Create a DRAFT release
/// 2. Update the release with additional metadata
/// 3. Publish the release (DRAFT -> PUBLISHED)
/// 4. Yank the release (PUBLISHED -> YANKED)
/// 5. Delete the release
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

    // Step 1: Create a DRAFT release
    println!("=== Step 1: Create DRAFT Release ===");
    // Use stable channel with simple version format
    let version = format!("0.0.{}", chrono::Utc::now().timestamp() % 10000);
    let release = Release::create(CreateReleaseRequest {
        version: version.clone(),
        channel: ReleaseChannel::Stable,
        product_id,
        name: Some(format!("v{}", version)),
        description: Some("Test release for lifecycle demo".to_string()),
        status: Some(ReleaseStatus::Draft),
        tag: Some(format!("v{}", version)),
        metadata: None,
    })
    .await?;

    println!("Created release:");
    println!("  ID: {}", release.id);
    println!("  Version: {}", release.version);
    println!("  Status: {:?}", release.status);

    // Step 2: Update the release
    println!("\n=== Step 2: Update Release ===");
    let mut metadata = HashMap::new();
    metadata.insert(
        "sha256".to_string(),
        serde_json::json!("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
    );
    metadata.insert(
        "changelog".to_string(),
        serde_json::json!("- Initial beta release\n- Core functionality implemented"),
    );

    let updated_release = release
        .update(UpdateReleaseRequest {
            description: Some("Beta release ready for testing".to_string()),
            metadata: Some(metadata),
            ..Default::default()
        })
        .await?;

    println!("Updated release:");
    println!("  Description: {:?}", updated_release.description);
    println!("  Metadata: {:?}", updated_release.metadata);

    // Step 3: Publish the release
    println!("\n=== Step 3: Publish Release ===");
    let published_release = updated_release.publish().await?;
    println!("Published release:");
    println!("  Status: {:?}", published_release.status);

    // Step 4: Yank the release
    println!("\n=== Step 4: Yank Release ===");
    let yanked_release = published_release.yank().await?;
    println!("Yanked release:");
    println!("  Status: {:?}", yanked_release.status);
    println!("  Yanked At: {:?}", yanked_release.yanked_at);

    // Step 5: Delete the release
    println!("\n=== Step 5: Delete Release ===");
    yanked_release.delete().await?;
    println!("Release deleted successfully!");

    println!("\n=== Release Lifecycle Complete ===");

    Ok(())
}
