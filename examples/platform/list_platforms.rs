use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    platform::{ListPlatformsOptions, Platform},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    let options = ListPlatformsOptions {
        limit: Some(25),
        ..Default::default()
    };

    match Platform::list(Some(options)).await {
        Ok(platforms) => {
            println!("Found {} platforms:", platforms.len());
            for platform in platforms {
                let name = platform.name.unwrap_or_else(|| "(unnamed)".to_string());
                println!("  - {} (key: {}) [ID: {}]", name, platform.key, platform.id);
            }
        }
        Err(e) => {
            println!("Failed to list platforms: {e:?}");
        }
    }

    Ok(())
}
