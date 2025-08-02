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

    // Configure with admin token for management operations
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    }).expect("Failed to set config");

    // Get the license ID from environment variable
    let license_id = env::var("KEYGEN_LICENSE_ID")
        .expect("KEYGEN_LICENSE_ID must be set (the license to attach entitlements to)");
    
    // Get entitlement IDs from environment variable (comma-separated)
    let entitlement_ids_str = env::var("KEYGEN_ENTITLEMENT_IDS")
        .expect("KEYGEN_ENTITLEMENT_IDS must be set (comma-separated list of entitlement IDs)");
    
    let entitlement_ids: Vec<String> = entitlement_ids_str
        .split(',')
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();

    if entitlement_ids.is_empty() {
        eprintln!("No entitlement IDs provided");
        eprintln!("Set KEYGEN_ENTITLEMENT_IDS with comma-separated entitlement IDs");
        return Ok(());
    }

    // Get the license
    let license = License::get(&license_id).await?;
    
    println!("License: {} ({})", license.id, license.key);

    // Display current entitlements before attaching
    let current_entitlements = license.entitlements(None).await?;
    println!("Current entitlements: {}", current_entitlements.len());

    // Attach the entitlements
    license.attach_entitlements(&entitlement_ids).await?;

    println!("Attached {} entitlement(s)", entitlement_ids.len());

    // Verify the entitlements were attached
    let updated_entitlements = license.entitlements(None).await?;
    
    println!("Total entitlements: {}", updated_entitlements.len());
    for entitlement in &updated_entitlements {
        println!("  {} ({})", entitlement.code, entitlement.id);
    }


    Ok(())
}