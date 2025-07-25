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
    })?;

    // List all users
    match user::list(None).await {
        Ok(result) => {
            println!("✅ Found {} users:", result.users.len());
            
            // Display pagination metadata if available
            if let Some(meta) = &result.meta {
                if let Some(total) = meta.get("total") {
                    println!("Total users: {}", total);
                }
                if let Some(page) = meta.get("page") {
                    println!("Current page: {}", page);
                }
            }
            
            println!("\nUser list:");
            for user in result.users {
                println!("  ID: {}", user.id);
                println!("  Email: {}", user.email);
                println!("  Full Name: {:?}", user.full_name);
                println!("  Role: {:?}", user.role);
                println!("  Created: {}", user.created);
                if let Some(metadata) = user.metadata {
                    if !metadata.is_empty() {
                        println!("  Metadata: {:?}", metadata);
                    }
                }
                println!("  ---");
            }
        }
        Err(e) => {
            println!("❌ Failed to list users: {:?}", e);
        }
    }

    Ok(())
}
