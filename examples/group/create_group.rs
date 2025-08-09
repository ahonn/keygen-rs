use chrono::Utc;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    group::{CreateGroupRequest, Group},
};
use std::{collections::HashMap, env};

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

    // Create metadata for the group
    let mut metadata = HashMap::new();
    metadata.insert(
        "department".to_string(),
        serde_json::Value::String("Engineering".to_string()),
    );
    metadata.insert(
        "region".to_string(),
        serde_json::Value::String("US-East".to_string()),
    );
    metadata.insert(
        "tier".to_string(),
        serde_json::Value::String("premium".to_string()),
    );

    // Create a new group
    let timestamp = Utc::now().timestamp();
    let request = CreateGroupRequest {
        name: format!("Engineering Team {timestamp}"),
        max_users: Some(25),
        max_licenses: Some(100),
        max_machines: Some(200),
        metadata: Some(metadata),
    };

    match Group::create(request).await {
        Ok(group) => {
            println!("Group created successfully!");
            println!("ID: {}", group.id);
            println!("Name: {}", group.name);
            println!("Max Users: {:?}", group.max_users);
            println!("Max Licenses: {:?}", group.max_licenses);
            println!("Max Machines: {:?}", group.max_machines);
            println!("Metadata: {:?}", group.metadata);
            println!("Created: {}", group.created);
        }
        Err(e) => {
            println!("Failed to create group: {e:?}");
        }
    }

    Ok(())
}
