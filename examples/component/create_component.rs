use dotenv::dotenv;
use keygen_rs::{
    component::{Component, CreateComponentRequest},
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

    // Get required machine ID for component creation
    let machine_id = env::var("KEYGEN_MACHINE_ID").expect("KEYGEN_MACHINE_ID must be set");

    // Create metadata for the component
    let mut metadata = HashMap::new();
    metadata.insert(
        "component_type".to_string(),
        serde_json::Value::String("GPU".to_string()),
    );
    metadata.insert(
        "model".to_string(),
        serde_json::Value::String("RTX 4090".to_string()),
    );
    metadata.insert(
        "driver_version".to_string(),
        serde_json::Value::String("528.49".to_string()),
    );

    // Create a new component using the builder pattern
    let fingerprint = format!("gpu-{}", uuid::Uuid::new_v4());
    let request = CreateComponentRequest::new(fingerprint, "Primary GPU".to_string(), machine_id)
        .with_metadata(metadata);

    match Component::create(request).await {
        Ok(component) => {
            println!("Component created: {}", component.id);
            println!("ID: {}", component.id);
            println!("Fingerprint: {}", component.fingerprint);
            println!("Name: {}", component.name);
            println!("Machine ID: {:?}", component.machine_id);
            println!("Metadata: {:?}", component.metadata);
            println!("Created: {}", component.created);
        }
        Err(e) => {
            println!("Failed to create component: {e:?}");
        }
    }

    Ok(())
}
