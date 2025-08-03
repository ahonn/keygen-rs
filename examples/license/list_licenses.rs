use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::License,
};
use std::env;

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
    })
    .expect("Failed to set config");

    // List all licenses (basic usage)
    match License::list(None).await {
        Ok(licenses) => {
            println!("Found {} licenses:", licenses.len());
            for license in licenses {
                println!("  ID: {}", license.id);
                println!("  Key: {}", license.key);
                println!("  Status: {:?}", license.status);
                println!("  Uses: {:?}", license.uses);
                println!("  Max Machines: {:?}", license.max_machines);
                println!("  Max Cores: {:?}", license.max_cores);
                println!("  Max Uses: {:?}", license.max_uses);
                println!("  Max Processes: {:?}", license.max_processes);
                println!("  Protected: {:?}", license.protected);
                println!("  Suspended: {:?}", license.suspended);
                println!("  Expiry: {:?}", license.expiry);
                println!("  Relationships:");
                println!("    Account ID: {:?}", license.account_id);
                println!("    Product ID: {:?}", license.product_id);
                println!("    Group ID: {:?}", license.group_id);
                println!("    Owner ID: {:?}", license.owner_id);
                println!("    Policy ID: {:?}", license.policy);
                println!("  ---");
            }
        }
        Err(e) => {
            println!("Failed to list licenses: {e:?}");
        }
    }

    Ok(())
}
