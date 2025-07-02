use keygen_rs::{
    config::{self, KeygenConfig},
    product,
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // List all products
    match product::list().await {
        Ok(products) => {
            println!("✅ Found {} products:", products.len());
            for product in products {
                println!("  ID: {}", product.id);
                println!("  Name: {}", product.name);
                println!("  Distribution Strategy: {:?}", product.distribution_strategy);
                println!("  Platforms: {:?}", product.platforms);
                println!("  Created: {}", product.created);
                println!("  ---");
            }
        },
        Err(e) => {
            println!("❌ Failed to list products: {:?}", e);
        }
    }

    Ok(())
}