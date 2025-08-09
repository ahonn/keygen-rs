use keygen_rs::{
    config::{self, KeygenConfig},
    environment::Environment,
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

    // First, get the environment to show details before deletion
    let environment = match Environment::get(&environment_id).await {
        Ok(env) => env,
        Err(e) => {
            println!("Failed to get environment: {e:?}");
            return Ok(());
        }
    };

    println!("Environment to delete:");
    println!("  ID: {}", environment.id);
    println!("  Name: {}", environment.name);
    println!("  Code: {}", environment.code);
    println!("  Isolation Strategy: {:?}", environment.isolation_strategy);

    // Confirm deletion (in a real app, you might want to add user confirmation)
    println!("\nWarning: This will permanently delete the environment and queue all associated resources for deletion!");

    // Delete the environment
    match environment.delete().await {
        Ok(()) => {
            println!(
                "✅ Environment '{}' deleted successfully!",
                environment.name
            );
            println!("All associated resources have been queued for deletion.");
        }
        Err(e) => {
            println!("❌ Failed to delete environment: {e:?}");
        }
    }

    Ok(())
}
