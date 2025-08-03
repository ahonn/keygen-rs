use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::{License, LicenseListOptions},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    let account = match env::var("KEYGEN_ACCOUNT_ID") {
        Ok(account) => account,
        Err(_) => {
            println!("❌ KEYGEN_ACCOUNT_ID environment variable is required");
            println!("💡 Please set the following environment variables:");
            println!("   export KEYGEN_ACCOUNT_ID=your-account-id");
            println!("   export KEYGEN_TOKEN=your-admin-token");
            println!("\n📚 This example demonstrates the fixed license pagination functionality.");
            println!("🔧 The key changes made:");
            println!("   - Fixed pagination parameters from 'page' to 'page[number]'");
            println!("   - Added support for 'page[size]' parameter");
            println!("   - Removed unsupported 'offset' parameter");
            println!("   - All pagination now follows Keygen API standards");
            return Ok(());
        }
    };

    let token = match env::var("KEYGEN_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            println!("❌ KEYGEN_TOKEN environment variable is required for management operations");
            return Ok(());
        }
    };

    config::set_config(KeygenConfig {
        api_url: "https://api.keygen.sh".to_string(),
        account: account.clone(),
        token: Some(token),
        ..KeygenConfig::default()
    })?;

    println!("🔑 License Pagination Examples\n");

    // Example 1: List all licenses with default pagination
    println!("📋 Example 1: List all licenses (default pagination)");
    match License::list(None).await {
        Ok(licenses) => {
            println!("✅ Found {} licenses", licenses.len());
            for license in &licenses[..3.min(licenses.len())] {
                println!("  - {} ({})", license.id, license.key);
            }
            if licenses.len() > 3 {
                println!("  ... and {} more", licenses.len() - 3);
            }
        }
        Err(e) => {
            println!("❌ Failed to list licenses: {:?}", e);
        }
    }

    println!("\n---\n");

    // Example 2: List licenses with a specific limit
    println!("📋 Example 2: List licenses with limit (5 results)");
    let options_with_limit = LicenseListOptions {
        limit: Some(5),
        ..Default::default()
    };
    match License::list(Some(&options_with_limit)).await {
        Ok(licenses) => {
            println!("✅ Found {} licenses (limited to 5)", licenses.len());
            for license in licenses {
                println!("  - {} ({})", license.id, license.key);
            }
        }
        Err(e) => {
            println!("❌ Failed to list licenses: {:?}", e);
        }
    }

    println!("\n---\n");

    // Example 3: List licenses with page-based pagination
    println!("📋 Example 3: List licenses with page-based pagination (page 1, 3 per page)");
    let options_with_page = LicenseListOptions {
        page_number: Some(1),
        page_size: Some(3),
        ..Default::default()
    };
    match License::list(Some(&options_with_page)).await {
        Ok(licenses) => {
            println!("✅ Found {} licenses on page 1:", licenses.len());
            for license in licenses {
                println!("  - {} ({})", license.id, license.key);
            }
        }
        Err(e) => {
            println!("❌ Failed to list licenses: {:?}", e);
        }
    }

    println!("\n---\n");

    // Example 4: List licenses with page-based pagination (page 2)
    println!("📋 Example 4: List licenses with page-based pagination (page 2, 3 per page)");
    let options_page_2 = LicenseListOptions {
        page_number: Some(2),
        page_size: Some(3),
        ..Default::default()
    };
    match License::list(Some(&options_page_2)).await {
        Ok(licenses) => {
            println!("✅ Found {} licenses on page 2:", licenses.len());
            for license in licenses {
                println!("  - {} ({})", license.id, license.key);
            }
        }
        Err(e) => {
            println!("❌ Failed to list licenses: {:?}", e);
        }
    }

    println!("\n---\n");

    // Example 5: Combine pagination with filters
    println!("📋 Example 5: List licenses with filters and pagination");
    let options_filtered = LicenseListOptions {
        status: Some("ACTIVE".to_string()),
        page_number: Some(1),
        page_size: Some(2),
        ..Default::default()
    };
    match License::list(Some(&options_filtered)).await {
        Ok(licenses) => {
            println!(
                "✅ Found {} active licenses on page 1 (2 per page):",
                licenses.len()
            );
            for license in licenses {
                println!(
                    "  - {} ({}) - Status: {:?}",
                    license.id, license.key, license.status
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to list licenses: {:?}", e);
        }
    }

    println!("\n🎉 Pagination examples completed!");
    Ok(())
}
