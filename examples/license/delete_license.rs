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
    }).expect("Failed to set config");

    // Get license ID from environment or command line
    let license_id = env::var("KEYGEN_LICENSE_ID").expect("KEYGEN_LICENSE_ID must be set");

    // Fetch the license first to confirm it exists
    let license = License::get(&license_id).await?;
    println!("📋 License to be deleted:");
    println!("  ID: {}", license.id);
    println!("  Key: {}", license.key);
    println!("  Name: {:?}", license.name);
    println!("  Status: {:?}", license.status);

    // Prompt for confirmation (in production, you might want a more robust confirmation)
    println!("\n⚠️  WARNING: This action cannot be undone!");
    println!("Are you sure you want to delete this license? (type 'yes' to confirm)");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    if input.trim().to_lowercase() == "yes" {
        // Delete the license
        license.delete().await?;
        println!("\n✅ License deleted successfully!");
    } else {
        println!("\n❌ License deletion cancelled.");
    }

    Ok(())
}
