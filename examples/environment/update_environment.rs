use keygen_rs::{
    config::{self, KeygenConfig},
    environment::{Environment, UpdateEnvironmentRequest},
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

    // Get environment ID from command line argument or environment variable
    let environment_id = env::args()
        .nth(1)
        .or_else(|| env::var("KEYGEN_ENVIRONMENT_ID").ok())
        .expect(
            "Please provide an environment ID as argument or set KEYGEN_ENVIRONMENT_ID environment variable",
        );

    // First, get the existing environment
    let environment = match Environment::get(&environment_id).await {
        Ok(env) => env,
        Err(e) => {
            println!("Failed to get environment: {e:?}");
            return Ok(());
        }
    };

    println!("Current environment details:");
    println!("  ID: {}", environment.id);
    println!("  Name: {}", environment.name);
    println!("  Code: {}", environment.code);

    // Update the environment with new name
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let update_request = UpdateEnvironmentRequest {
        name: Some(format!("{} (Updated {})", environment.name, timestamp)),
        code: None, // Keep existing code
    };

    match environment.update(update_request).await {
        Ok(updated_environment) => {
            println!("\nEnvironment updated successfully!");
            println!("  ID: {}", updated_environment.id);
            println!("  Name: {}", updated_environment.name);
            println!("  Code: {}", updated_environment.code);
            println!(
                "  Isolation Strategy: {:?}",
                updated_environment.isolation_strategy
            );
            println!("  Updated: {}", updated_environment.updated);
        }
        Err(e) => {
            println!("Failed to update environment: {e:?}");
        }
    }

    Ok(())
}
