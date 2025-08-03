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
    })
    .expect("Failed to set config");

    // First validate to get the license
    let license = match keygen_rs::validate(&[], &[]).await {
        Ok(license) => license,
        Err(Error::LicenseNotActivated { license, .. }) => *license,
        Err(Error::ValidationFingerprintMissing { .. }) => keygen_rs::validate(&[], &[]).await?,
        Err(e) => return Err(e),
    };

    println!("License: {} ({})", license.id, license.key);

    // List all entitlements with pagination
    let mut all_entitlements = Vec::new();
    let mut page = 1;
    let limit = 50;

    loop {
        let pagination = PaginationOptions {
            limit: Some(limit),
            page_number: Some(page),
            page_size: Some(limit),
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
        println!("No entitlements found");
        return Ok(());
    }

    println!("Total entitlements: {}", all_entitlements.len());

    // Display entitlements
    for entitlement in &all_entitlements {
        println!("  {} ({})", entitlement.code, entitlement.id);
    }

    Ok(())
}
