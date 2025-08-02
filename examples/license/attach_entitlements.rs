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
        eprintln!("❌ No entitlement IDs provided");
        eprintln!("💡 Set KEYGEN_ENTITLEMENT_IDS with comma-separated entitlement IDs");
        return Ok(());
    }

    println!("🔍 Fetching license: {}", license_id);
    
    // Get the license
    let license = License::get(&license_id).await?;
    
    println!("✅ License found: {}", license.key);
    println!("  Name: {:?}", license.name.as_deref().unwrap_or("N/A"));
    println!("  Status: {:?}", license.status.as_deref().unwrap_or("N/A"));

    // Display current entitlements before attaching
    println!("\n📋 Current entitlements:");
    let current_entitlements = license.entitlements(None).await?;
    if current_entitlements.is_empty() {
        println!("  No entitlements currently attached");
    } else {
        for entitlement in &current_entitlements {
            println!("  - {} ({})", entitlement.code, entitlement.id);
        }
    }

    println!("\n🔗 Attaching entitlements to license...");
    println!("  Entitlement IDs: {:?}", entitlement_ids);

    // Attach the entitlements
    license.attach_entitlements(&entitlement_ids).await?;

    println!("✅ Successfully attached {} entitlement(s)!", entitlement_ids.len());

    // Verify the entitlements were attached
    println!("\n🔍 Verifying entitlements were attached...");
    let updated_entitlements = license.entitlements(None).await?;
    
    println!("📋 Updated entitlements list:");
    if updated_entitlements.is_empty() {
        println!("  No entitlements found (this might indicate an issue)");
    } else {
        for entitlement in &updated_entitlements {
            let is_newly_attached = entitlement_ids.contains(&entitlement.id);
            let marker = if is_newly_attached { "🆕" } else { "📌" };
            println!("  {} {} ({})", marker, entitlement.code, entitlement.id);
            if let Some(name) = &entitlement.name {
                println!("      Name: {}", name);
            }
        }
    }

    // Show usage example
    println!("\n{:=<60}", "");
    println!("Usage Example");
    println!("{:=<60}", "");
    println!("To validate this license with the attached entitlements, use:");
    println!();
    
    if !updated_entitlements.is_empty() {
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
    }

    // Summary
    println!("\n{:=<60}", "");
    println!("Summary");
    println!("{:=<60}", "");
    println!("License ID: {}", license.id);
    println!("Total entitlements after attach: {}", updated_entitlements.len());
    println!("Entitlements added: {}", entitlement_ids.len());

    Ok(())
}