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
        .or_else(|| env::var("KEYGEN_PRODUCT_ID").ok())
        .expect(
            "Please provide a product ID as argument or set KEYGEN_PRODUCT_ID environment variable",
        );

    // First, get the product to ensure it exists
    match Product::get(&product_id).await {
        Ok(product) => {
            println!("Found product:");
            println!("  ID: {}", product.id);
            println!("  Name: {}", product.name);
            println!("  Code: {:?}", product.code);

            // Delete the product
            match product.delete().await {
                Ok(()) => {
                    println!("Product deleted: {}", product.id);
                }
                Err(e) => {
                    println!("Failed to delete product: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("Failed to find product: {e:?}");
        }
    }

    Ok(())
}
