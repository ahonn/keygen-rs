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

    // Configure with Admin Token for management operations
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })
    .expect("Failed to set config");

    // Get license ID from environment or command line
    let license_id = env::var("KEYGEN_LICENSE_ID").expect("KEYGEN_LICENSE_ID must be set");

    // Fetch the license first
    let license = License::get(&license_id).await?;
    println!("Current License Details:");
    println!("  ID: {}", license.id);
    println!("  Key: {}", license.key);
    println!("  Name: {:?}", license.name);
    println!("  Status: {:?}", license.status);
    println!("  Expiry: {:?}", license.expiry);

    // Check if license is eligible for renewal
    if let Some(status) = &license.status {
        if status == "EXPIRED" || status == "EXPIRING_SOON" {
            println!("License is eligible for renewal");
        } else {
            println!(
                "Note: License status is '{}', but proceeding with renewal anyway.",
                status
            );
        }
    }

    // Renew the license
    let renewed_license = license.renew().await?;

    println!("License renewed: {}", renewed_license.id);
    println!("Renewed License Details:");
    println!("  ID: {}", renewed_license.id);
    println!("  Key: {}", renewed_license.key);
    println!("  Name: {:?}", renewed_license.name);
    println!("  Status: {:?}", renewed_license.status);
    println!("  Expiry: {:?}", renewed_license.expiry);

    // Show the extension period if expiry dates are available
    if let (Some(old_expiry), Some(new_expiry)) = (license.expiry, renewed_license.expiry) {
        let extension = new_expiry - old_expiry;
        println!("License extended by: {} days", extension.num_days());
    }

    Ok(())
}
