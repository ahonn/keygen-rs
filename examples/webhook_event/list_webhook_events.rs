use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    webhook::event::{WebhookEventListOptions, WebhookEventRecord},
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

    // List all webhook events
    let options = WebhookEventListOptions {
        limit: Some(20),
        page_size: Some(20),
        page_number: Some(1),
        event_type: None,
        status: None,
    };

    match WebhookEventRecord::list(Some(&options)).await {
        Ok(events) => {
            println!("Found {} webhook events:", events.len());
            for event in events {
                println!("  ID: {}", event.id);
                println!("  Event: {}", event.event);
                println!("  Status: {:?}", event.status);
                println!("  Endpoint: {}", event.endpoint);
                if let Some(code) = event.last_response_code {
                    println!("  Last Response Code: {}", code);
                }
                if let Some(ref body) = event.last_response_body {
                    let preview = if body.len() > 50 {
                        format!("{}...", &body[..50])
                    } else {
                        body.clone()
                    };
                    println!("  Last Response Body: {}", preview);
                }
                println!("  Created: {}", event.created);
                println!("  Updated: {}", event.updated);
                println!("  ---");
            }
        }
        Err(e) => {
            println!("Failed to list webhook events: {e:?}");
        }
    }

    Ok(())
}
