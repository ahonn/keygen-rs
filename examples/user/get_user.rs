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

    // Get user ID from command line argument
    let user_id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example get_user <user_id>");

    // Get specific user
    match user::get(&user_id).await {
        Ok(user) => {
            println!("Action completed:");
            println!("ID: {}", user.id);
            println!("Email: {}", user.email);
            println!("First Name: {:?}", user.first_name);
            println!("Last Name: {:?}", user.last_name);
            println!("Full Name: {:?}", user.full_name);
            println!("Role: {:?}", user.role);
            println!("Status: {:?}", user.status);
            println!("Created: {}", user.created);
            println!("Updated: {}", user.updated);

            if let Some(permissions) = user.permissions {
                println!("Permissions: {:?}", permissions);
            }

            if let Some(metadata) = user.metadata {
                if !metadata.is_empty() {
                    println!("Metadata: {:?}", metadata);
                }
            }
        }
        Err(e) => {
            println!("Failed to get user: {:?}", e);
        }
    }

    Ok(())
}
