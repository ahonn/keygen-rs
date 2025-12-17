use keygen_rs::{
    channel::{Channel, ListChannelsOptions},
    config::{self, KeygenConfig},
    errors::Error,
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

    let options = ListChannelsOptions {
        limit: Some(25),
        ..Default::default()
    };

    match Channel::list(Some(options)).await {
        Ok(channels) => {
            println!("Found {} channels:", channels.len());
            for channel in channels {
                let name = channel.name.unwrap_or_else(|| "(unnamed)".to_string());
                println!("  - {} (key: {}) [ID: {}]", name, channel.key, channel.id);
            }
        }
        Err(e) => {
            println!("Failed to list channels: {e:?}");
        }
    }

    Ok(())
}
