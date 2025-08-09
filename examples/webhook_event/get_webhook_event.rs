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

    // Get the webhook event details
    match WebhookEventRecord::get(&event_id).await {
        Ok(event) => {
            println!("Webhook event found: {}", event.id);
            println!("  ID: {}", event.id);
            println!("  Event: {}", event.event);
            println!("  Status: {:?}", event.status);
            println!("  Endpoint: {}", event.endpoint);
            println!("  Account ID: {:?}", event.account_id);
            if let Some(code) = event.last_response_code {
                println!("  Last Response Code: {}", code);
            }
            if let Some(ref body) = event.last_response_body {
                println!("  Last Response Body: {}", body);
            }
            println!(
                "  Payload: {}",
                serde_json::to_string_pretty(&event.payload).unwrap_or_else(|_| "N/A".to_string())
            );
            println!("  Created: {}", event.created);
            println!("  Updated: {}", event.updated);

            // Show status helper information
            println!("  Status Checks:");
            println!("    Is Delivered: {}", event.is_delivered());
            println!("    Is Failed: {}", event.is_failed());
            println!("    Is Pending: {}", event.is_pending());
        }
        Err(e) => {
            println!("Failed to get webhook event: {e:?}");
        }
    }

    Ok(())
}
