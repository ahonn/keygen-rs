use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    release::{Release, UpdateReleaseRequest},
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

    // Get release ID from environment
    let release_id = env::var("KEYGEN_RELEASE_ID").expect("KEYGEN_RELEASE_ID must be set");

    // First, get the release
    let release = Release::get(&release_id).await?;
    println!("Current release:");
    println!("  Version: {}", release.version);
    println!("  Name: {:?}", release.name);
    println!("  Description: {:?}", release.description);

    // Update the release
    let mut metadata = HashMap::new();
    metadata.insert(
        "updatedAt".to_string(),
        serde_json::json!(chrono::Utc::now().to_rfc3339()),
    );
    metadata.insert(
        "changelog".to_string(),
        serde_json::json!("- Fixed critical bug\n- Improved performance"),
    );

    let update_request = UpdateReleaseRequest {
        name: Some(format!(
            "{} (Updated)",
            release.name.clone().unwrap_or_default()
        )),
        description: Some("Updated release with additional fixes".to_string()),
        metadata: Some(metadata),
        ..Default::default()
    };

    match release.update(update_request).await {
        Ok(updated_release) => {
            println!("\nRelease updated successfully!");
            println!("  ID: {}", updated_release.id);
            println!("  Version: {}", updated_release.version);
            println!("  Name: {:?}", updated_release.name);
            println!("  Description: {:?}", updated_release.description);
            println!("  Metadata: {:?}", updated_release.metadata);
            println!("  Updated: {}", updated_release.updated);
        }
        Err(e) => {
            println!("Failed to update release: {e:?}");
        }
    }

    Ok(())
}
