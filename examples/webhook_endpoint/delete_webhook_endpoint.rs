use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    webhook::endpoint::WebhookEndpoint,
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

    // Get webhook endpoint ID from command line arguments or environment
    let endpoint_id = env::args()
        .nth(1)
        .or_else(|| env::var("WEBHOOK_ENDPOINT_ID").ok())
        .expect("Please provide a webhook endpoint ID as an argument or set WEBHOOK_ENDPOINT_ID");

    // First, get the existing webhook endpoint to confirm it exists
    match WebhookEndpoint::get(&endpoint_id).await {
        Ok(endpoint) => {
            println!("Found webhook endpoint:");
            println!("  ID: {}", endpoint.id);
            println!("  URL: {}", endpoint.url);
            println!("  Subscriptions: {:?}", endpoint.subscriptions);

            // Delete the webhook endpoint
            match endpoint.delete().await {
                Ok(()) => {
                    println!("Webhook endpoint deleted: {}", endpoint.id);
                }
                Err(e) => {
                    println!("Failed to delete webhook endpoint: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("Failed to find webhook endpoint: {e:?}");
        }
    }

    Ok(())
}
