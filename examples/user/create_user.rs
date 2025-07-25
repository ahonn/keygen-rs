use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    user::{self, CreateUserRequest, UserRole},
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
    })?;

    // Create metadata for the user
    let mut metadata = HashMap::new();
    metadata.insert(
        "department".to_string(),
        serde_json::Value::String("Engineering".to_string()),
    );
    metadata.insert(
        "employee_id".to_string(),
        serde_json::Value::String("EMP001".to_string()),
    );

    // Create a new user with timestamp-based email to avoid conflicts
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let request = CreateUserRequest {
        email: format!("user-{}@example.com", timestamp),
        first_name: Some("John".to_string()),
        last_name: Some("Doe".to_string()),
        role: Some(UserRole::User),
        permissions: None,
        metadata: Some(metadata),
    };

    match user::create(request).await {
        Ok(user) => {
            println!("✅ User created successfully!");
            println!("ID: {}", user.id);
            println!("Email: {}", user.email);
            println!("Full Name: {:?}", user.full_name);
            println!("Role: {:?}", user.role);
            if let Some(metadata) = user.metadata {
                println!("Metadata: {:?}", metadata);
            }
        }
        Err(e) => {
            println!("❌ Failed to create user: {:?}", e);
        }
    }

    Ok(())
}
