use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    machine::{Machine, MachineUpdateRequest},
};
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // Update a machine
    let machine_id = env::var("MACHINE_ID").expect("MACHINE_ID must be set");

    // First get the machine
    let machine = Machine::get(&machine_id).await?;

    // Update with new information
    let mut metadata = HashMap::new();
    metadata.insert(
        "environment".to_string(),
        serde_json::Value::String("staging".to_string()),
    );
    metadata.insert(
        "updated_at".to_string(),
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );

    let request = MachineUpdateRequest {
        name: Some("Updated Machine Name".to_string()),
        platform: Some("linux".to_string()),
        hostname: Some("updated-server".to_string()),
        ip: Some("192.168.1.101".to_string()),
        cores: Some(8),
        metadata: Some(metadata),
    };

    match machine.update(request).await {
        Ok(updated_machine) => {
            println!("✅ Machine updated successfully!");
            println!("ID: {}", updated_machine.id);
            println!("Fingerprint: {}", updated_machine.fingerprint);
            println!("Name: {:?}", updated_machine.name);
            println!("Platform: {:?}", updated_machine.platform);
            println!("Hostname: {:?}", updated_machine.hostname);
            println!("IP: {:?}", updated_machine.ip);
            println!("Cores: {:?}", updated_machine.cores);
            println!("Metadata: {:?}", updated_machine.metadata);
            println!("Updated: {}", updated_machine.updated);
        }
        Err(e) => {
            println!("❌ Failed to update machine: {:?}", e);
        }
    }

    Ok(())
}
