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
            println!("KEYGEN_ACCOUNT_ID environment variable is required");
            println!("Please set the following environment variables:");
            println!("   export KEYGEN_ACCOUNT_ID=your-account-id");
            println!("   export KEYGEN_TOKEN=your-admin-token");
            return Ok(());
        }
    };

    let token = match env::var("KEYGEN_TOKEN") {
        Ok(token) => token,
        Err(_) => {
            println!("KEYGEN_TOKEN environment variable is required for management operations");
            return Ok(());
        }
    };

    config::set_config(KeygenConfig {
        api_url: "https://api.keygen.sh".to_string(),
        account: account.clone(),
        token: Some(token),
        ..KeygenConfig::default()
    })?;

    // Example 1: List all licenses with default pagination
    match License::list(None).await {
        Ok(licenses) => {
            println!("Found {} licenses", licenses.len());
            for license in &licenses[..3.min(licenses.len())] {
                println!("  {} ({})", license.id, license.key);
            }
            if licenses.len() > 3 {
                println!("  ... and {} more", licenses.len() - 3);
            }
        }
        Err(e) => {
            println!("Failed to list licenses: {e:?}");
        }
    }

    // Example 2: List licenses with a specific limit
    let options_with_limit = LicenseListOptions {
        limit: Some(5),
        ..Default::default()
    };
    match License::list(Some(&options_with_limit)).await {
        Ok(licenses) => {
            println!("Found {} licenses (limited to 5)", licenses.len());
            for license in licenses {
                println!("  {} ({})", license.id, license.key);
            }
        }
        Err(e) => {
            println!("Failed to list licenses: {e:?}");
        }
    }

    // Example 3: List licenses with page-based pagination
    let options_with_page = LicenseListOptions {
        page_number: Some(1),
        page_size: Some(3),
        ..Default::default()
    };
    match License::list(Some(&options_with_page)).await {
        Ok(licenses) => {
            println!("Found {} licenses on page 1:", licenses.len());
            for license in licenses {
                println!("  {} ({})", license.id, license.key);
            }
        }
        Err(e) => {
            println!("Failed to list licenses: {e:?}");
        }
    }
    Ok(())
}
