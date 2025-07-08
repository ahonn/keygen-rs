use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    token::Token,
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
    });

    // Get token ID from command line argument or environment variable
    let token_id = env::args()
        .nth(1)
        .or_else(|| env::var("TOKEN_ID").ok())
        .expect("Please provide a token ID as argument or set TOKEN_ID environment variable");

    // First, get the token to ensure it exists
    match Token::get(&token_id).await {
        Ok(token) => {
            println!("🔑 Found token:");
            println!("  ID: {}", token.id);
            println!("  Kind: {:?}", token.kind);
            println!("  Name: {:?}", token.name);
            println!("  Permissions: {:?}", token.permissions);

            // Revoke the token
            match token.revoke().await {
                Ok(()) => {
                    println!("\n✅ Token revoked successfully!");
                    println!("⚠️  This token is now invalid and cannot be used for API requests.");
                }
                Err(e) => {
                    println!("❌ Failed to revoke token: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to find token: {:?}", e);
        }
    }

    Ok(())
}
