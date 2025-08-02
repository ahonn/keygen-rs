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
        .expect("KEYGEN_LICENSE_ID must be set (the license to detach entitlements from)");
    
    // Get entitlement IDs from environment variable (comma-separated)
    let entitlement_ids_str = env::var("KEYGEN_ENTITLEMENT_IDS")
        .expect("KEYGEN_ENTITLEMENT_IDS must be set (comma-separated list of entitlement IDs to detach)");
    
    let entitlement_ids: Vec<String> = entitlement_ids_str
        .split(',')
        .map(|id| id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();

    if entitlement_ids.is_empty() {
        eprintln!("❌ No entitlement IDs provided");
        eprintln!("💡 Set KEYGEN_ENTITLEMENT_IDS with comma-separated entitlement IDs to detach");
        return Ok(());
    }

    println!("🔍 Fetching license: {}", license_id);
    
    // Get the license
    let license = License::get(&license_id).await?;
    
    println!("✅ License found: {}", license.key);
    println!("  Name: {:?}", license.name.as_deref().unwrap_or("N/A"));
    println!("  Status: {:?}", license.status.as_deref().unwrap_or("N/A"));

    // Display current entitlements before detaching
    println!("\n📋 Current entitlements:");
    let current_entitlements = license.entitlements(None).await?;
    if current_entitlements.is_empty() {
        println!("  No entitlements currently attached");
        println!("💡 Nothing to detach!");
        return Ok(());
    } else {
        for entitlement in &current_entitlements {
            let will_be_detached = entitlement_ids.contains(&entitlement.id);
            let marker = if will_be_detached { "🗑️" } else { "📌" };
            println!("  {} {} ({})", marker, entitlement.code, entitlement.id);
            if let Some(name) = &entitlement.name {
                println!("      Name: {}", name);
            }
        }
    }

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
        println!("\n⚠️  Warning: Some entitlements are not attached to this license:");
        for id in &missing_ids {
            println!("  - {}", id);
        }
    }

    if ids_to_detach.is_empty() {
        println!("\n❌ No valid entitlements to detach!");
        return Ok(());
    }

    println!("\n🔓 Detaching entitlements from license...");
    println!("  Entitlement IDs to detach: {:?}", ids_to_detach);

    // Detach the entitlements
    license.detach_entitlements(&ids_to_detach).await?;

    println!("✅ Successfully detached {} entitlement(s)!", ids_to_detach.len());

    // Verify the entitlements were detached
    println!("\n🔍 Verifying entitlements were detached...");
    let updated_entitlements = license.entitlements(None).await?;
    
    println!("📋 Updated entitlements list:");
    if updated_entitlements.is_empty() {
        println!("  No entitlements attached (all were detached or none existed)");
    } else {
        for entitlement in &updated_entitlements {
            println!("  📌 {} ({})", entitlement.code, entitlement.id);
            if let Some(name) = &entitlement.name {
                println!("      Name: {}", name);
            }
        }
    }

    // Show usage example if entitlements remain
    if !updated_entitlements.is_empty() {
        println!("\n{:=<60}", "");
        println!("Usage Example");
        println!("{:=<60}", "");
        println!("To validate this license with the remaining entitlements, use:");
        println!();
        
        let entitlement_codes: Vec<String> = updated_entitlements
            .iter()
            .map(|e| e.code.clone())
            .collect();

        println!("```rust");
        println!("let license = keygen_rs::validate(");
        println!("    &[fingerprint],");
        println!("    &{:?}", entitlement_codes);
        println!(").await?;");
        println!("```");
    } else {
        println!("\n💡 This license now has no entitlements attached.");
        println!("   Validation will not check for specific entitlements.");
    }

    // Summary
    println!("\n{:=<60}", "");
    println!("Summary");
    println!("{:=<60}", "");
    println!("License ID: {}", license.id);
    println!("Entitlements before detach: {}", current_entitlements.len());
    println!("Entitlements after detach: {}", updated_entitlements.len());
    println!("Entitlements removed: {}", ids_to_detach.len());
    
    if !missing_ids.is_empty() {
        println!("Entitlements not found: {}", missing_ids.len());
    }

    Ok(())
}