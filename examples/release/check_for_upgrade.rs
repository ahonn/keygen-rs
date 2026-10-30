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

    // Set up configuration with License Key (no admin token required)
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        public_key: env::var("KEYGEN_PUBLIC_KEY").ok(),
        ..KeygenConfig::default()
    })?;

    let current_version =
        env::var("KEYGEN_CURRENT_VERSION").unwrap_or_else(|_| "1.0.0".to_string());

    // Product comes from the config, channel defaults to stable
    match Release::upgrade_from(&current_version, None).await {
        Ok(Some(release)) => {
            println!(
                "Upgrade available: {} -> {}",
                current_version, release.version
            );
            println!("  ID: {}", release.id);
            println!("  Channel: {:?}", release.channel);
            println!("  Name: {:?}", release.name);
        }
        Ok(None) => {
            // Also returned for unknown versions and products the license cannot access
            println!("No upgrade available for {current_version}");
        }
        Err(e) => {
            println!("Failed to check for upgrade: {e:?}");
        }
    }

    Ok(())
}
