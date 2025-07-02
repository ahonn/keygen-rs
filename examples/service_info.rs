use keygen_rs::config::{self, KeygenConfig};
use keygen_rs::service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Configure the SDK
    config::set_config(KeygenConfig {
        api_url: std::env::var("KEYGEN_API_URL")
            .unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: std::env::var("KEYGEN_ACCOUNT")
            .expect("KEYGEN_ACCOUNT environment variable is required"),
        #[cfg(feature = "token")]
        token: Some(
            std::env::var("KEYGEN_TOKEN")
                .expect("KEYGEN_TOKEN environment variable is required"),
        ),
        ..KeygenConfig::default()
    });

    println!("Pinging Keygen service...");

    // First try the simple ping
    match service::ping().await {
        Ok(ping_resp) => {
            println!("✅ Ping Response:");
            println!("  Message: {}", ping_resp.message);
            println!("  Version: {:?}", ping_resp.version);
            println!("  Timestamp: {:?}", ping_resp.timestamp);
        }
        Err(e) => {
            println!("❌ Ping failed: {}", e);
        }
    }

    println!("\nGetting detailed service information...");

    match service::get_service_info().await {
        Ok(info) => {
            println!("✅ Service Info Retrieved:");
            println!("  Message: {:?}", info.message);
            println!("  API Version: {:?}", info.api_version);
            println!("  Timestamp: {:?}", info.timestamp);
            println!("  Key Headers:");
            for (key, value) in &info.headers {
                if key.contains("version") || key.contains("date") || key.contains("server") {
                    println!("    {}: {}", key, value);
                }
            }

            // Check specific feature support
            println!("\n🔍 Feature Support:");
            println!(
                "  Product Code (v1.8+): {}",
                service::supports_feature(&info, "1.8")
            );

            // Use the convenience function
            match service::supports_product_code().await {
                Ok(supported) => {
                    println!("  Product Code Support: {}", supported);
                }
                Err(e) => {
                    println!("  Error checking product code support: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get service info: {}", e);
        }
    }

    Ok(())
}