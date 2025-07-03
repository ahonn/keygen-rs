use keygen_rs::{
    config::{self, KeygenConfig},
    token::Token,
    errors::Error,
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

    // Get the token details
    match Token::get(&token_id).await {
        Ok(token) => {
            println!("✅ Token found!");
            println!("  ID: {}", token.id);
            println!("  Kind: {:?}", token.kind);
            println!("  Name: {:?}", token.name);
            println!("  Permissions: {:?}", token.permissions);
            println!("  Expiry: {:?}", token.expiry);
            if token.is_expired() {
                println!("  ⚠️  Status: EXPIRED");
            } else {
                println!("  ✅ Status: Active");
            }
            println!("  Metadata: {:?}", token.metadata);
            println!("  Created: {}", token.created);
            println!("  Updated: {}", token.updated);
            
            // Note: Token value is not returned in GET requests for security
            println!("\n📝 Note: Token value is not returned for security reasons. It's only shown during generation/regeneration.");
        },
        Err(e) => {
            println!("❌ Failed to get token: {:?}", e);
        }
    }

    Ok(())
}