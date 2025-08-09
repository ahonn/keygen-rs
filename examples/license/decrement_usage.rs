use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::License,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    // Set up configuration with Admin Token (required for decrement operation)
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(
            env::var("KEYGEN_ADMIN_TOKEN")
                .expect("KEYGEN_ADMIN_TOKEN must be set (admin token required)"),
        ),
        ..KeygenConfig::default()
    })
    .expect("Failed to set config");

    // Get license ID from environment
    let license_id = env::var("KEYGEN_LICENSE_ID").expect("KEYGEN_LICENSE_ID must be set");

    // Retrieve and update the license
    let license = License::get(&license_id).await?;
    println!("Current License: {}", license.key);
    println!("  Uses: {:?}", license.uses);
    println!("  Max Uses: {:?}", license.max_uses);

    let updated_license = license.decrement_usage().await?;

    println!("Usage decremented: {}", updated_license.id);
    println!("  Previous Uses: {:?}", license.uses);
    println!("  Current Uses: {:?}", updated_license.uses);
    println!("  Max Uses: {:?}", updated_license.max_uses);

    // Show the change
    if let (Some(old_uses), Some(new_uses)) = (license.uses, updated_license.uses) {
        println!("  Change: -{}", old_uses - new_uses);
    }

    Ok(())
}
