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
    });

    // Get license ID from environment or command line
    let license_id = env::var("KEYGEN_LICENSE_ID").expect("KEYGEN_LICENSE_ID must be set");

    // Fetch the license first
    let license = License::get(&license_id).await?;
    println!("📋 Current License Details:");
    println!("  ID: {}", license.id);
    println!("  Key: {}", license.key);
    println!("  Name: {:?}", license.name);
    println!("  Status: {:?}", license.status);

    // Prompt for confirmation
    println!("\n⚠️  WARNING: Revoking a license will permanently invalidate it!");
    println!("This action is typically used for licenses that have been compromised or misused.");
    println!("Are you sure you want to revoke this license? (type 'yes' to confirm)");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    if input.trim().to_lowercase() == "yes" {
        // Revoke the license
        license.revoke().await?;

        println!("\n✅ License revoked successfully!");
        println!("📋 Revoked License:");
        println!("  ID: {}", license.id);
        println!("  Key: {}", license.key);
        println!("\n🚫 This license has been permanently revoked and can no longer be used for validation or activation.");
    } else {
        println!("\n❌ License revocation cancelled.");
    }

    Ok(())
}
