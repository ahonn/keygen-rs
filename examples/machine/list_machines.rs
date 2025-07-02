use keygen_rs::{
    config::{self, KeygenConfig},
    machine,
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // List all machines
    match machine::list().await {
        Ok(machines) => {
            println!("✅ Found {} machines:", machines.len());
            for machine in machines {
                println!("  ID: {}", machine.id);
                println!("  Fingerprint: {}", machine.fingerprint);
                println!("  Name: {:?}", machine.name);
                println!("  Platform: {:?}", machine.platform);
                println!("  Hostname: {:?}", machine.hostname);
                println!("  IP: {:?}", machine.ip);
                println!("  Cores: {:?}", machine.cores);
                println!("  Created: {}", machine.created);
                if let Some(last_heartbeat) = machine.last_heartbeat_at {
                    println!("  Last Heartbeat: {}", last_heartbeat);
                }
                println!("  ---");
            }
        },
        Err(e) => {
            println!("❌ Failed to list machines: {:?}", e);
        }
    }

    Ok(())
}