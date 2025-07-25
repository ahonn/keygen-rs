use dotenv::dotenv;
use keygen_rs::config::{self, KeygenConfig};
use keygen_rs::service;
use std::env;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv().ok();

    // Load configuration from environment
    let api_url =
        env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string());
    let account = env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set");

    // Configure the client
    config::set_config(KeygenConfig {
        api_url,
        account,
        ..KeygenConfig::default()
    })?;

    println!("🏥 Performing Keygen service health check...\n");

    let start_time = Instant::now();

    match service::ping().await {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            println!("✅ Service is healthy!");
            println!("   Response time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
            println!("   Message: {}", response.message);

            if let Some(version) = &response.version {
                println!("   API Version: {}", version);
            }

            if let Some(timestamp) = &response.timestamp {
                println!("   Server Timestamp: {}", timestamp);
            }

            // Determine health status based on response time
            let health_status = match elapsed.as_millis() {
                0..=100 => "🟢 Excellent",
                101..=300 => "🟡 Good",
                301..=1000 => "🟠 Fair",
                _ => "🔴 Poor",
            };

            println!("   Health Status: {}", health_status);

            // Additional diagnostics
            println!("\n📋 Diagnostic Information:");

            match service::get_service_info().await {
                Ok(info) => {
                    println!("   ✓ Service introspection successful");

                    if let Some(api_version) = &info.api_version {
                        println!("   ✓ API version detected: {}", api_version);

                        // Check for known versions and their capabilities
                        if service::supports_feature(&info, "1.8") {
                            println!("   ✓ Modern features supported (v1.8+)");
                        } else if service::supports_feature(&info, "1.5") {
                            println!("   ⚠ Limited features (v1.5-1.7)");
                        } else {
                            println!("   ⚠ Legacy version detected");
                        }
                    } else {
                        println!("   ⚠ API version not detected");
                    }

                    // Check headers for additional info
                    if info.headers.contains_key("keygen-version") {
                        println!("   ✓ Keygen version header present");
                    }

                    if info.headers.contains_key("x-ratelimit-limit") {
                        println!("   ✓ Rate limiting headers present");
                    }
                }
                Err(e) => {
                    println!("   ✗ Service introspection failed: {}", e);
                }
            }

            // Test feature-specific capabilities
            println!("\n🧪 Feature Testing:");

            match service::supports_product_code().await {
                Ok(true) => println!("   ✓ Product codes supported"),
                Ok(false) => println!("   ✗ Product codes not supported"),
                Err(e) => println!("   ⚠ Could not test product code support: {}", e),
            }

            println!("\n🎯 Overall Status: Service is operational");
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            println!("❌ Service health check failed!");
            println!("   Error: {}", e);
            println!(
                "   Time to failure: {:.2}ms",
                elapsed.as_secs_f64() * 1000.0
            );

            // Provide troubleshooting hints
            println!("\n🔧 Troubleshooting:");
            println!("   • Check if KEYGEN_ACCOUNT is set correctly");
            println!("   • Verify network connectivity");
            println!(
                "   • Confirm API URL is reachable: {}",
                env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string())
            );

            return Err(e.into());
        }
    }

    Ok(())
}
