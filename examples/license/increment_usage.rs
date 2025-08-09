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

    // Create a license instance with the ID
    let license = License {
        id: license_id,
        scheme: None,
        key: env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set"),
        name: None,
        expiry: None,
        status: None,
        uses: None,
        max_machines: None,
        max_cores: None,
        max_uses: None,
        max_processes: None,
        max_users: None,
        protected: None,
        suspended: None,
        permissions: None,
        policy: None,
        metadata: std::collections::HashMap::new(),
        account_id: None,
        product_id: None,
        group_id: None,
        owner_id: None,
    };

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
