use keygen_rs::{
    config::{self, KeygenConfig},
    product::Product,
    errors::Error,
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
            println!("  ID: {}", product.id);
            println!("  Name: {}", product.name);
            println!("  Code: {:?}", product.code);
            
            // Generate a product token
            match product.generate_token().await {
                Ok(token) => {
                    println!("\n✅ Product token generated successfully!");
                    println!("🔑 Token: {}", token);
                    println!("\n⚠️  Keep this token secure! It can be used to authenticate API requests for this product.");
                },
                Err(e) => {
                    println!("❌ Failed to generate product token: {:?}", e);
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to find product: {:?}", e);
        }
    }

    Ok(())
}