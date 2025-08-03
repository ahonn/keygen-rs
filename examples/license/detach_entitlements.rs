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
    })
    .expect("Failed to set config");

    // Get the license ID from environment variable
    let license_id = env::var("KEYGEN_LICENSE_ID")
        .expect("KEYGEN_LICENSE_ID must be set (the license to detach entitlements from)");

    // Get entitlement IDs from environment variable (comma-separated)
    let entitlement_ids_str = env::var("KEYGEN_ENTITLEMENT_IDS").expect(
        "KEYGEN_ENTITLEMENT_IDS must be set (comma-separated list of entitlement IDs to detach)",
    );

    let entitlement_ids: Vec<String> = entitlement_ids_str
        .split(',')
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();

    if entitlement_ids.is_empty() {
        eprintln!("No entitlement IDs provided");
        eprintln!("Set KEYGEN_ENTITLEMENT_IDS with comma-separated entitlement IDs to detach");
        return Ok(());
    }

    // Get the license
    let license = License::get(&license_id).await?;

    println!("License: {} ({})", license.id, license.key);

    // Display current entitlements before detaching
    let current_entitlements = license.entitlements(None).await?;
    if current_entitlements.is_empty() {
        println!("No entitlements currently attached");
        return Ok(());
    }

    println!("Current entitlements: {}", current_entitlements.len());

    // Check if the entitlements to detach actually exist
    let existing_ids: Vec<String> = current_entitlements.iter().map(|e| e.id.clone()).collect();
    let ids_to_detach: Vec<String> = entitlement_ids
        .iter()
        .filter(|id| existing_ids.contains(id))
        .cloned()
        .collect();

    let missing_ids: Vec<String> = entitlement_ids
        .iter()
        .filter(|id| !existing_ids.contains(id))
        .cloned()
        .collect();

    if !missing_ids.is_empty() {
        println!("Warning: Some entitlements are not attached to this license:");
        for id in &missing_ids {
            println!("  {id}");
        }
    }

    if ids_to_detach.is_empty() {
        println!("No valid entitlements to detach");
        return Ok(());
    }

    // Detach the entitlements
    license.detach_entitlements(&ids_to_detach).await?;

    println!("Detached {} entitlement(s)", ids_to_detach.len());

    // Verify the entitlements were detached
    let updated_entitlements = license.entitlements(None).await?;

    println!("Remaining entitlements: {}", updated_entitlements.len());
    for entitlement in &updated_entitlements {
        println!("  {} ({})", entitlement.code, entitlement.id);
    }

    Ok(())
}
