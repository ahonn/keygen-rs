use keygen_rs::{
    config::{self, KeygenConfig},
    environment::{CreateEnvironmentRequest, Environment, IsolationStrategy},
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // Create a new environment with timestamp-based code to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let request = CreateEnvironmentRequest {
        name: format!("Test Environment {timestamp}"),
        code: format!("test-env-{timestamp}"),
        isolation_strategy: Some(IsolationStrategy::Isolated),
    };

    match Environment::create(request).await {
        Ok(environment) => {
            println!("Environment created successfully!");
            println!("  ID: {}", environment.id);
            println!("  Name: {}", environment.name);
            println!("  Code: {}", environment.code);
            println!("  Isolation Strategy: {:?}", environment.isolation_strategy);
            println!("  Created: {}", environment.created);
            println!("  Account ID: {:?}", environment.account_id);
        }
        Err(e) => {
            println!("Failed to create environment: {e:?}");
        }
    }

    Ok(())
}
