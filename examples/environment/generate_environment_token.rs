use keygen_rs::{
    config::{self, KeygenConfig},
    environment::{CreateEnvironmentTokenRequest, Environment},
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

    // First, get the environment
    let environment = match Environment::get(&environment_id).await {
        Ok(env) => env,
        Err(e) => {
            println!("Failed to get environment: {e:?}");
            return Ok(());
        }
    };

    println!("Generating token for environment:");
    println!("  ID: {}", environment.id);
    println!("  Name: {}", environment.name);
    println!("  Code: {}", environment.code);

    // Create token request with custom settings
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let token_request = CreateEnvironmentTokenRequest {
        name: Some(format!("Environment Token {}", timestamp)),
        expiry: None, // No expiration
        permissions: Some(vec![
            "environment.read".to_string(),
            "license.read".to_string(),
            "license.create".to_string(),
            "machine.read".to_string(),
        ]),
    };

    // Generate the environment token
    match environment.generate_token(Some(token_request)).await {
        Ok(token) => {
            println!("\n✅ Environment token generated successfully!");
            println!("  Token ID: {}", token.id);
            println!("  Token Name: {:?}", token.name);
            println!("  Token: {}", token.token);
            println!("  Expiry: {:?}", token.expiry);
            println!("  Permissions: {:?}", token.permissions);
            println!("  Created: {}", token.created);
            println!("  Environment ID: {}", token.environment_id);
            println!("  Account ID: {:?}", token.account_id);

            println!("\n🔐 You can now use this token to authenticate API requests for this environment:");
            println!("  KEYGEN_ENVIRONMENT_TOKEN={}", token.token);
        }
        Err(e) => {
            println!("❌ Failed to generate environment token: {e:?}");
        }
    }

    Ok(())
}
