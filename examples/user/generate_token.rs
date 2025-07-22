use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    user,
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
    });

    // Get user ID from command line argument
    let user_id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example generate_token <user_id>");

    // Generate user token
    match user::generate_token(&user_id, Some("API Access Token"), None).await {
        Ok(token) => {
            println!("✅ Token generated successfully!");
            println!("Token ID: {}", token.id);
            println!("Token Name: {}", token.name);
            println!("Token: {}", token.token);
            println!("Created: {}", token.created);
            if let Some(expiry) = token.expiry {
                println!("Expires: {}", expiry);
            } else {
                println!("Expires: Never");
            }
        }
        Err(e) => {
            println!("❌ Failed to generate token: {:?}", e);
        }
    }

    Ok(())
}
