use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    product::{DistributionStrategy, Platform, Product, UpdateProductRequest},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // Get product ID from command line argument or environment variable
    let product_id = env::args()
        .nth(1)
        .or_else(|| env::var("PRODUCT_ID").ok())
        .expect("Please provide a product ID as argument or set PRODUCT_ID environment variable");

    // First, get the product
    match Product::get(&product_id).await {
        Ok(product) => {
            println!("📦 Found product:");
            println!("  Current Name: {}", product.name);
            println!(
                "  Current Distribution Strategy: {:?}",
                product.distribution_strategy
            );
            println!("  Current Platforms: {:?}", product.platforms);

            // Update the product
            let update_request = UpdateProductRequest {
                name: Some(format!("{} (Updated)", product.name)),
                code: None, // Keep existing code
                distribution_strategy: Some(DistributionStrategy::Open),
                url: Some("https://updated-example.com".to_string()),
                platforms: Some(vec![
                    Platform::Windows,
                    Platform::MacOs,
                    Platform::Linux,
                    Platform::Web,
                ]),
                permissions: None,
                metadata: None,
            };

            match product.update(update_request).await {
                Ok(updated_product) => {
                    println!("\n✅ Product updated successfully!");
                    println!("  New Name: {}", updated_product.name);
                    println!(
                        "  New Distribution Strategy: {:?}",
                        updated_product.distribution_strategy
                    );
                    println!("  New URL: {:?}", updated_product.url);
                    println!("  New Platforms: {:?}", updated_product.platforms);
                }
                Err(e) => {
                    println!("❌ Failed to update product: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to find product: {:?}", e);
        }
    }

    Ok(())
}
