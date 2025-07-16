use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::PaginationOptions,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    // Configure for license operations
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        ..KeygenConfig::default()
    });

    // First validate to get the license
    println!("🔍 Fetching license information...");
    let license = match keygen_rs::validate(&[], &[]).await {
        Ok(license) => license,
        Err(Error::LicenseNotActivated { license, .. }) => license,
        Err(Error::ValidationFingerprintMissing { .. }) => keygen_rs::validate(&[], &[]).await?,
        Err(e) => return Err(e),
    };

    println!("✅ License found: {}", license.key);
    println!(
        "  License Name: {:?}",
        license.name.as_deref().unwrap_or("N/A")
    );
    println!("  Status: {:?}", license.status.as_deref().unwrap_or("N/A"));

    // List all entitlements with pagination
    let mut all_entitlements = Vec::new();
    let mut page = 1;
    let limit = 50;

    loop {
        println!("\n📋 Fetching entitlements (page {})...", page);

        let pagination = PaginationOptions {
            limit: Some(limit),
            page: None,
            offset: Some((page - 1) * limit),
        };

        let entitlements = license.entitlements(Some(&pagination)).await?;

        if entitlements.is_empty() {
            break;
        }

        let entitlements_len = entitlements.len();
        all_entitlements.extend(entitlements);

        // If we got less than the limit, we've reached the last page
        if (entitlements_len as i32) < limit {
            break;
        }

        page += 1;
    }

    if all_entitlements.is_empty() {
        println!("\n❌ No entitlements found for this license.");
        println!("💡 Tip: Entitlements are features or permissions granted to a license.");
        println!(
            "   They can be used to enable/disable specific functionality in your application."
        );
        return Ok(());
    }

    println!(
        "\n✅ Found {} entitlement(s) for this license:",
        all_entitlements.len()
    );

    // Display detailed information for each entitlement
    for (i, entitlement) in all_entitlements.iter().enumerate() {
        println!("\n{:=<60}", "");
        println!("Entitlement #{}", i + 1);
        println!("{:=<60}", "");
        println!("  ID:           {}", entitlement.id);
        println!("  Code:         {}", entitlement.code);
        println!(
            "  Name:         {:?}",
            entitlement.name.as_deref().unwrap_or("N/A")
        );
        println!("  Created:      {:?}", entitlement.created);
        println!("  Updated:      {:?}", entitlement.updated);
    }

    // Show how to validate with entitlements
    println!("\n{:=<60}", "");
    println!("Usage Example");
    println!("{:=<60}", "");
    println!("To validate this license with specific entitlements, use:");
    println!();

    if !all_entitlements.is_empty() {
        let entitlement_codes: Vec<String> = all_entitlements
            .iter()
            .take(2) // Show first 2 as example
            .map(|e| e.code.clone())
            .collect();

        println!("```rust");
        println!("let license = keygen_rs::validate(");
        println!("    &[fingerprint],");
        println!("    &{:?}", entitlement_codes);
        println!(").await?;");
        println!("```");

        println!(
            "\nThis will validate that the license has access to these specific entitlements."
        );
    }

    // Summary
    println!("\n{:=<60}", "");
    println!("Summary");
    println!("{:=<60}", "");
    println!("Total entitlements: {}", all_entitlements.len());

    // Group by name/type if patterns exist
    let mut entitlement_types = std::collections::HashMap::new();
    for entitlement in &all_entitlements {
        let key = entitlement
            .name
            .clone()
            .unwrap_or_else(|| "Unnamed".to_string());
        *entitlement_types.entry(key).or_insert(0) += 1;
    }

    if entitlement_types.len() > 1 {
        println!("\nEntitlement types:");
        for (name, count) in entitlement_types {
            println!("  {}: {}", name, count);
        }
    }

    Ok(())
}
