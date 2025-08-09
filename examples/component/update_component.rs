use dotenv::dotenv;
use keygen_rs::{
    component::{Component, UpdateComponentRequest},
    config::{self, KeygenConfig},
    errors::Error,
};
use std::{collections::HashMap, env};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // Get the component ID from environment variable
    let component_id = env::var("KEYGEN_COMPONENT_ID").expect("KEYGEN_COMPONENT_ID must be set");

    // First, retrieve the current component
    let component = match Component::get(&component_id).await {
        Ok(component) => component,
        Err(e) => {
            println!("Failed to retrieve component: {e:?}");
            return Ok(());
        }
    };

    // Create updated metadata
    let mut metadata = HashMap::new();
    metadata.insert(
        "component_type".to_string(),
        serde_json::Value::String("GPU".to_string()),
    );
    metadata.insert(
        "model".to_string(),
        serde_json::Value::String("RTX 4090 Ti".to_string()), // Updated model
    );
    metadata.insert(
        "driver_version".to_string(),
        serde_json::Value::String("535.98".to_string()), // Updated driver
    );
    metadata.insert(
        "last_maintenance".to_string(),
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );

    // Create update request using the builder pattern
    let update_request = UpdateComponentRequest::new()
        .with_name("Updated Primary GPU".to_string())
        .with_metadata(metadata);

    match component.update(update_request).await {
        Ok(updated_component) => {
            println!("Component updated: {}", updated_component.id);
            println!("ID: {}", updated_component.id);
            println!("Name: {}", updated_component.name);
            println!("Metadata: {:?}", updated_component.metadata);
            println!("Updated: {}", updated_component.updated);
        }
        Err(e) => {
            println!("Failed to update component: {e:?}");
        }
    }

    Ok(())
}
