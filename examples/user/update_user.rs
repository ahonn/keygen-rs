use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    user::{self, UpdateUserRequest, UserRole},
};
use std::collections::HashMap;
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
        .expect("Usage: cargo run --example update_user <user_id>");

    // Update metadata for the user
    let mut metadata = HashMap::new();
    metadata.insert(
        "department".to_string(),
        serde_json::Value::String("Product Management".to_string()),
    );
    metadata.insert(
        "last_updated".to_string(),
        serde_json::Value::String("2024-01-01T00:00:00Z".to_string()),
    );

    // Update user
    let request = UpdateUserRequest {
        email: None,
        first_name: Some("Jane".to_string()),
        last_name: Some("Smith".to_string()),
        role: Some(UserRole::Developer),
        password: None,
        metadata: Some(metadata),
    };

    match user::update(&user_id, request).await {
        Ok(user) => {
            println!("✅ User updated successfully!");
            println!("ID: {}", user.id);
            println!("Email: {}", user.email);
            println!("Full Name: {:?}", user.full_name);
            println!("Role: {:?}", user.role);
            if let Some(metadata) = user.metadata {
                println!("Metadata: {:?}", metadata);
            }
        }
        Err(e) => {
            println!("❌ Failed to update user: {:?}", e);
        }
    }

    Ok(())
}
