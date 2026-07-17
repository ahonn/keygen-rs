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

    // Set up configuration with License Key
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        ..KeygenConfig::default()
    })
    .expect("Failed to set config");

    // Get license ID from environment
    let license_id = env::var("KEYGEN_LICENSE_ID").expect("KEYGEN_LICENSE_ID must be set");

    let license = License::from_id(&license_id);

    match license.increment_usage().await {
        Ok(updated_license) => {
            println!("Usage incremented: {}", updated_license.id);
            println!("  Uses: {:?}", updated_license.uses);
            println!("  Max Uses: {:?}", updated_license.max_uses);

            // Show usage percentage if limits are set
            if let (Some(uses), Some(max_uses)) = (updated_license.uses, updated_license.max_uses) {
                let percentage = (uses as f32 / max_uses as f32) * 100.0;
                println!("  Usage: {}/{} ({:.1}%)", uses, max_uses, percentage);
            }
        }
        Err(e) => {
            eprintln!("Failed to increment usage: {:?}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
