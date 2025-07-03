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

    // List all tokens
    match Token::list(None).await {
        Ok(tokens) => {
            println!("✅ Found {} tokens:", tokens.len());
            for token in tokens {
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
                println!("  Created: {}", token.created);
                println!("  Updated: {}", token.updated);
                println!("  ---");
            }
        },
        Err(e) => {
            println!("❌ Failed to list tokens: {:?}", e);
        }
    }

    Ok(())
}