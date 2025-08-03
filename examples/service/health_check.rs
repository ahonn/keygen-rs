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

    let start_time = Instant::now();

    match service::ping().await {
        Ok(response) => {
            let elapsed = start_time.elapsed();
            println!("Service is healthy");
            println!("Response time: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
            println!("Message: {}", response.message);

            if let Some(version) = &response.version {
                println!("API Version: {version}");
            }

            if let Some(timestamp) = &response.timestamp {
                println!("Server Timestamp: {timestamp}");
            }
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            println!("Service health check failed: {e}");
            println!("Time to failure: {:.2}ms", elapsed.as_secs_f64() * 1000.0);
            return Err(e.into());
        }
    }

    Ok(())
}
