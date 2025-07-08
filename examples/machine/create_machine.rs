use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    machine::{Machine, MachineCreateRequest},
    errors::Error,
};
use std::env;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // Create a new machine
    let license_id = env::var("LICENSE_ID").expect("LICENSE_ID must be set");
    let fingerprint = machine_uid::get().unwrap_or("example-fingerprint".into());
    
    let mut metadata = HashMap::new();
    metadata.insert("environment".to_string(), serde_json::Value::String("production".to_string()));
    metadata.insert("version".to_string(), serde_json::Value::String("1.0.0".to_string()));
    
    let request = MachineCreateRequest {
        fingerprint,
        name: Some("My Machine".to_string()),
        platform: Some("linux".to_string()),
        hostname: Some("my-server".to_string()),
        ip: Some("192.168.1.100".to_string()),
        cores: Some(4),
        metadata: Some(metadata),
        license_id,
    };

    match Machine::create(request).await {
        Ok(machine) => {
            println!("✅ Machine created successfully!");
            println!("ID: {}", machine.id);
            println!("Fingerprint: {}", machine.fingerprint);
            println!("Name: {:?}", machine.name);
            println!("Platform: {:?}", machine.platform);
            println!("Hostname: {:?}", machine.hostname);
            println!("IP: {:?}", machine.ip);
            println!("Cores: {:?}", machine.cores);
            println!("Metadata: {:?}", machine.metadata);
            println!("Created: {}", machine.created);
        },
        Err(e) => {
            println!("❌ Failed to create machine: {:?}", e);
        }
    }

    Ok(())
}