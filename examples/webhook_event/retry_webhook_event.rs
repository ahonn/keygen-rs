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

    // First, get the existing webhook event
    match WebhookEventRecord::get(&event_id).await {
        Ok(event) => {
            println!("Found webhook event:");
            println!("  ID: {}", event.id);
            println!("  Event: {}", event.event);
            println!("  Status: {:?}", event.status);
            println!("  Endpoint: {}", event.endpoint);
            if let Some(code) = event.last_response_code {
                println!("  Last Response Code: {}", code);
            }

            // Check if the event can be retried
            if event.is_delivered() {
                println!("Note: Event is already delivered, but retrying anyway.");
            } else if event.is_failed() {
                println!("Event has failed, retrying...");
            } else if event.is_pending() {
                println!("Event is currently pending, retrying...");
            }

            // Retry the webhook event
            match event.retry().await {
                Ok(retried_event) => {
                    println!("Webhook event retry initiated: {}", retried_event.id);
                    println!("  New Status: {:?}", retried_event.status);
                    println!("  Updated: {}", retried_event.updated);
                    println!("  Is Pending: {}", retried_event.is_pending());
                }
                Err(e) => {
                    println!("Failed to retry webhook event: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("Failed to find webhook event: {e:?}");
        }
    }

    Ok(())
}
