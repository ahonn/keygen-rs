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

    // Get environment ID or code from command line argument or environment variable
    let environment_id = env::args()
        .nth(1)
        .or_else(|| env::var("KEYGEN_ENVIRONMENT_ID").ok())
        .expect(
            "Please provide an environment ID/code as argument or set KEYGEN_ENVIRONMENT_ID environment variable",
        );

    // Get the environment by ID or code
    match Environment::get(&environment_id).await {
        Ok(environment) => {
            println!("Environment found:");
            println!("  ID: {}", environment.id);
            println!("  Name: {}", environment.name);
            println!("  Code: {}", environment.code);
            println!("  Isolation Strategy: {:?}", environment.isolation_strategy);
            println!("  Created: {}", environment.created);
            println!("  Updated: {}", environment.updated);
            println!("  Relationships:");
            println!("    Account ID: {:?}", environment.account_id);
        }
        Err(e) => {
            println!("Failed to get environment: {e:?}");
        }
    }

    Ok(())
}
