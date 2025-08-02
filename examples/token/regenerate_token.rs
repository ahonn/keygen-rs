use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    token::{RegenerateTokenRequest, Token},
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
    })?;

    // Get token ID from command line argument or environment variable
    let token_id = env::args()
        .nth(1)
        .or_else(|| env::var("KEYGEN_TOKEN_ID").ok())
        .expect("Please provide a token ID as argument or set KEYGEN_TOKEN_ID environment variable");

    // First, get the token
    match Token::get(&token_id).await {
        Ok(token) => {
            println!("🔄 Found token:");
            println!("  Current Name: {:?}", token.name);
            println!("  Current Kind: {:?}", token.kind);
            println!("  Current Permissions: {:?}", token.permissions);

            // Regenerate the token with updated permissions
            let current_name = token.name.clone().unwrap_or("Token".to_string());
            let regenerate_request = RegenerateTokenRequest {
                name: Some(format!("{} (Regenerated)", current_name)),
                expiry: None, // Keep no expiration
                permissions: Some(vec![
                    "product.read".to_string(),
                    "license.read".to_string(),
                    "license.validate".to_string(),
                    "user.read".to_string(),
                ]),
                metadata: None,
            };

            match token.regenerate(regenerate_request).await {
                Ok(regenerated_token) => {
                    println!("\n✅ Token regenerated successfully!");
                    println!("  New Name: {:?}", regenerated_token.name);
                    println!("  New Permissions: {:?}", regenerated_token.permissions);
                    if let Some(token_value) = regenerated_token.get_token() {
                        println!("  🔑 New Token: {}", token_value);
                        println!("\n⚠️  Keep this new token secure! The old token is now invalid.");
                    }
                    println!("  Updated: {}", regenerated_token.updated);
                }
                Err(e) => {
                    println!("Failed to regenerate token: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Failed to find token: {:?}", e);
        }
    }

    Ok(())
}
