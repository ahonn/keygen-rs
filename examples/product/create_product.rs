use keygen_rs::{
    config::{self, KeygenConfig},
    product::{Product, CreateProductRequest, DistributionStrategy, Platform},
    errors::Error,
};
use std::env;
use chrono;

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

    // Create a new product
    let request = CreateProductRequest {
        name: "My Software Product".to_string(),
        code: format!("my-software-v1-{}", chrono::Utc::now().timestamp()),
        url: Some("https://example.com".to_string()),
        distribution_strategy: Some(DistributionStrategy::Licensed),
        platforms: Some(vec![Platform::Windows, Platform::MacOs, Platform::Linux]),
        permissions: None,
        metadata: None,
    };

    match Product::create(request).await {
        Ok(product) => {
            println!("✅ Product created successfully!");
            println!("ID: {}", product.id);
            println!("Name: {}", product.name);
            println!("Code: {:?}", product.code);
            println!("Distribution Strategy: {:?}", product.distribution_strategy);
            println!("Platforms: {:?}", product.platforms);
        },
        Err(e) => {
            println!("❌ Failed to create product: {:?}", e);
        }
    }

    Ok(())
}