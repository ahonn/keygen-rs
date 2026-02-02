use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    release::Release,
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

    // Get release ID from environment
    let release_id = env::var("KEYGEN_RELEASE_ID").expect("KEYGEN_RELEASE_ID must be set");

    // First, get the release to check its current status
    let release = Release::get(&release_id).await?;
    println!("Current release status:");
    println!("  ID: {}", release.id);
    println!("  Version: {}", release.version);
    println!("  Status: {:?}", release.status);

    // Publish the release (DRAFT -> PUBLISHED)
    match release.publish().await {
        Ok(published_release) => {
            println!("\nRelease published successfully!");
            println!("  ID: {}", published_release.id);
            println!("  Version: {}", published_release.version);
            println!("  Status: {:?}", published_release.status);
            println!("  Updated: {}", published_release.updated);
        }
        Err(e) => {
            println!("\nFailed to publish release: {e:?}");
        }
    }

    Ok(())
}
