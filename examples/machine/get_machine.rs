use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    machine::Machine,
    errors::Error,
};
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
    });

    // Get a machine by ID
    let machine_id = env::var("MACHINE_ID").expect("MACHINE_ID must be set");
    
    match Machine::get(&machine_id).await {
        Ok(machine) => {
            println!("✅ Machine found!");
            println!("ID: {}", machine.id);
            println!("Fingerprint: {}", machine.fingerprint);
            println!("Name: {:?}", machine.name);
            println!("Platform: {:?}", machine.platform);
            println!("Hostname: {:?}", machine.hostname);
            println!("IP: {:?}", machine.ip);
            println!("Cores: {:?}", machine.cores);
            println!("Metadata: {:?}", machine.metadata);
            println!("Require Heartbeat: {}", machine.require_heartbeat);
            println!("Heartbeat Status: {}", machine.heartbeat_status);
            println!("Heartbeat Duration: {:?}", machine.heartbeat_duration);
            println!("Created: {}", machine.created);
            println!("Updated: {}", machine.updated);
        },
        Err(e) => {
            println!("❌ Failed to get machine: {:?}", e);
        }
    }

    Ok(())
}