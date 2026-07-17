use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::{LicenseValidationRequest, PaginationOptions},
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
    let license = keygen_rs::validate(&LicenseValidationRequest::default())
        .await?
        .into_license()?;

    println!("License: {} ({})", license.id, license.key);

    // List all entitlements with pagination
    let limit = 50;
    let pagination = PaginationOptions {
        limit: Some(limit),
        page_size: Some(limit),
        page_cursor: None,
    };
    let all_entitlements = license.entitlements(Some(&pagination)).await?.data;

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
