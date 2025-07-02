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

    // Reset a machine
    let machine_id = env::var("MACHINE_ID").expect("MACHINE_ID must be set (get from list_machines example)");
    
    match machine::reset(&machine_id).await {
        Ok(machine) => {
            println!("✅ Machine reset successfully!");
            println!("ID: {}", machine.id);
            println!("Fingerprint: {}", machine.fingerprint);
            println!("Name: {:?}", machine.name);
            println!("Last Heartbeat Reset: {:?}", machine.last_heartbeat_at);
        },
        Err(e) => {
            println!("❌ Failed to reset machine: {:?}", e);
        }
    }

    Ok(())
}