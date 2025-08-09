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

    // Get the webhook endpoint details
    match WebhookEndpoint::get(&endpoint_id).await {
        Ok(endpoint) => {
            println!("Webhook endpoint found: {}", endpoint.id);
            println!("  ID: {}", endpoint.id);
            println!("  URL: {}", endpoint.url);
            println!("  Subscriptions: {:?}", endpoint.subscriptions);
            println!("  Signature Algorithm: {:?}", endpoint.signature_algorithm);
            println!("  Account ID: {:?}", endpoint.account_id);
            println!("  Environment ID: {:?}", endpoint.environment_id);
            println!("  Created: {}", endpoint.created);
            println!("  Updated: {}", endpoint.updated);
        }
        Err(e) => {
            println!("Failed to get webhook endpoint: {e:?}");
        }
    }

    Ok(())
}
