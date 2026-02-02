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

    // Get release ID from environment or use a default for testing
    let release_id = env::var("KEYGEN_RELEASE_ID").expect("KEYGEN_RELEASE_ID must be set");

    // Get the release
    match Release::get(&release_id).await {
        Ok(release) => {
            println!("Release found!");
            println!("  ID: {}", release.id);
            println!("  Version: {}", release.version);
            println!("  Channel: {:?}", release.channel);
            println!("  Status: {:?}", release.status);
            println!("  Name: {:?}", release.name);
            println!("  Description: {:?}", release.description);
            println!("  Tag: {:?}", release.tag);
            println!("  Metadata: {:?}", release.metadata);
            println!("  Created: {}", release.created);
            println!("  Updated: {}", release.updated);
            println!("  Yanked At: {:?}", release.yanked_at);
            println!("  Semver: {:?}", release.semver);
            println!("  Relationships:");
            println!("    Product ID: {:?}", release.product_id);
            println!("    Account ID: {:?}", release.account_id);
        }
        Err(e) => {
            println!("Failed to get release: {e:?}");
        }
    }

    Ok(())
}
