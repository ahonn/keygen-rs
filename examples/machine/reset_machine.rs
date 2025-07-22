use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    machine::Machine,
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

    // Reset a machine
    let machine_id = env::var("MACHINE_ID").expect("MACHINE_ID must be set");

    // First get the machine
    let machine = Machine::get(&machine_id).await?;

    match machine.reset().await {
        Ok(updated_machine) => {
            println!("✅ Machine reset successfully!");
            println!("ID: {}", updated_machine.id);
            println!("Fingerprint: {}", updated_machine.fingerprint);
            println!("Name: {:?}", updated_machine.name);
            println!("Heartbeat Status: {}", updated_machine.heartbeat_status);
            println!(
                "Heartbeat Duration: {:?}",
                updated_machine.heartbeat_duration
            );
            println!("Updated: {}", updated_machine.updated);
        }
        Err(e) => {
            println!("❌ Failed to reset machine: {:?}", e);
        }
    }

    Ok(())
}
