use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    group::{Group, UpdateGroupRequest},
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

    // Get the group ID from environment variable
    let group_id = env::var("KEYGEN_GROUP_ID").expect("KEYGEN_GROUP_ID must be set");

    // First, retrieve the current group
    let group = match Group::get(&group_id).await {
        Ok(group) => {
            println!("Current group state:");
            println!("  Name: {}", group.name);
            println!("  Max Users: {:?}", group.max_users);
            println!("  Max Licenses: {:?}", group.max_licenses);
            println!("  Max Machines: {:?}", group.max_machines);
            println!();
            group
        }
        Err(e) => {
            println!("Failed to retrieve group: {e:?}");
            println!("Make sure KEYGEN_GROUP_ID is set to a valid group ID");
            return Ok(());
        }
    };

    // Create updated metadata
    let mut metadata = HashMap::new();
    metadata.insert(
        "department".to_string(),
        serde_json::Value::String("Engineering".to_string()),
    );
    metadata.insert(
        "region".to_string(),
        serde_json::Value::String("US-West".to_string()), // Updated region
    );
    metadata.insert(
        "tier".to_string(),
        serde_json::Value::String("enterprise".to_string()), // Upgraded tier
    );
    metadata.insert(
        "last_updated".to_string(),
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );

    // Create update request - only updating specific fields
    let update_request = UpdateGroupRequest {
        name: Some(format!("{} (Updated)", group.name)),
        max_users: Some(50),     // Increased user limit
        max_licenses: Some(200), // Increased license limit
        max_machines: None,      // Keep current value
        metadata: Some(metadata),
    };

    match group.update(update_request).await {
        Ok(updated_group) => {
            println!("Group updated successfully!");
            println!("ID: {}", updated_group.id);
            println!("Name: {} -> {}", group.name, updated_group.name);
            println!(
                "Max Users: {:?} -> {:?}",
                group.max_users, updated_group.max_users
            );
            println!(
                "Max Licenses: {:?} -> {:?}",
                group.max_licenses, updated_group.max_licenses
            );
            println!(
                "Max Machines: {:?} -> {:?}",
                group.max_machines, updated_group.max_machines
            );
            println!(
                "Updated: {}",
                updated_group.updated.format("%Y-%m-%d %H:%M:%S UTC")
            );

            if let Some(metadata) = &updated_group.metadata {
                println!("Updated Metadata:");
                for (key, value) in metadata {
                    println!("  {}: {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("Failed to update group: {e:?}");
        }
    }

    Ok(())
}
