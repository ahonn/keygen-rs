use dotenv::dotenv;
use keygen_rs::config::{self, KeygenConfig};
use keygen_rs::service;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();
    
    // Load configuration from environment
    let api_url = env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string());
    let account = env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set");

    // Configure the client
    config::set_config(KeygenConfig {
        api_url,
        account,
        ..KeygenConfig::default()
    });

    println!("🏓 Pinging Keygen service...\n");

    // Basic ping
    match service::ping().await {
        Ok(ping_response) => {
            println!("✅ Service is healthy!");
            println!("   Message: {}", ping_response.message);
            if let Some(version) = &ping_response.version {
                println!("   Version: {}", version);
            }
            if let Some(timestamp) = &ping_response.timestamp {
                println!("   Timestamp: {}", timestamp);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to ping service: {}", e);
            return Err(e.into());
        }
    }

    println!("\n📊 Getting detailed service information...\n");

    // Get detailed service info
    match service::get_service_info().await {
        Ok(service_info) => {
            println!("✅ Service information retrieved!");
            
            if let Some(api_version) = &service_info.api_version {
                println!("   API Version: {}", api_version);
            }
            
            if let Some(timestamp) = &service_info.timestamp {
                println!("   Server Time: {}", timestamp);
            }
            
            if let Some(message) = &service_info.message {
                println!("   Message: {}", message);
            }

            println!("   Headers received:");
            for (key, value) in &service_info.headers {
                if key.starts_with("keygen") || key.starts_with("x-") || key == "date" {
                    println!("     {}: {}", key, value);
                }
            }

            // Check feature support
            println!("\n🔍 Feature support checks:");
            
            println!("   Product codes (v1.8+): {}", 
                service::supports_feature(&service_info, "1.8"));
            
            println!("   Modern features (v1.5+): {}", 
                service::supports_feature(&service_info, "1.5"));
        }
        Err(e) => {
            eprintln!("❌ Failed to get service info: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🚀 Checking specific feature support...\n");

    // Check if product code field is supported
    match service::supports_product_code().await {
        Ok(supports) => {
            if supports {
                println!("✅ Product codes are supported by this Keygen instance");
            } else {
                println!("⚠️  Product codes are not supported (requires API v1.8+)");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to check product code support: {}", e);
        }
    }

    Ok(())
}