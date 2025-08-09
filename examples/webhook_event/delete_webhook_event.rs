use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    webhook::event::WebhookEventRecord,
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

    // Get webhook event ID from command line arguments or environment
    let event_id = env::args()
        .nth(1)
        .or_else(|| env::var("WEBHOOK_EVENT_ID").ok())
        .expect("Please provide a webhook event ID as an argument or set WEBHOOK_EVENT_ID");

    // First, get the existing webhook event to confirm it exists
    match WebhookEventRecord::get(&event_id).await {
        Ok(event) => {
            println!("Found webhook event:");
            println!("  ID: {}", event.id);
            println!("  Event: {}", event.event);
            println!("  Status: {:?}", event.status);
            println!("  Endpoint: {}", event.endpoint);
            println!("  Created: {}", event.created);

            // Delete the webhook event
            match event.delete().await {
                Ok(()) => {
                    println!("Webhook event deleted: {}", event.id);
                }
                Err(e) => {
                    println!("Failed to delete webhook event: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("Failed to find webhook event: {e:?}");
        }
    }

    Ok(())
}
