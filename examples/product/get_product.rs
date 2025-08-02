use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    product::Product,
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
    })?;

    // Get product ID from command line argument or environment variable
    let product_id = env::args()
        .nth(1)
        .or_else(|| env::var("PRODUCT_ID").ok())
        .expect("Please provide a product ID as argument or set PRODUCT_ID environment variable");

    // Get the product details
    match Product::get(&product_id).await {
        Ok(product) => {
            println!("Product found: {}", product.id);
            println!("  ID: {}", product.id);
            println!("  Name: {}", product.name);
            println!("  Code: {:?}", product.code);
            println!(
                "  Distribution Strategy: {:?}",
                product.distribution_strategy
            );
            println!("  URL: {:?}", product.url);
            println!("  Platforms: {:?}", product.platforms);
            println!("  Permissions: {:?}", product.permissions);
            println!("  Metadata: {:?}", product.metadata);
            println!("  Created: {}", product.created);
            println!("  Updated: {}", product.updated);
        }
        Err(e) => {
            println!("Failed to get product: {:?}", e);
        }
    }

    Ok(())
}
