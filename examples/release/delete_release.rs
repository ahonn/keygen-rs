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

    // First, get the release to confirm it exists
    let release = Release::get(&release_id).await?;
    println!("Found release to delete:");
    println!("  ID: {}", release.id);
    println!("  Version: {}", release.version);
    println!("  Status: {:?}", release.status);

    // Delete the release
    match release.delete().await {
        Ok(()) => {
            println!("\nRelease deleted successfully!");
        }
        Err(e) => {
            println!("\nFailed to delete release: {e:?}");
        }
    }

    Ok(())
}
