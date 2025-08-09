use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    webhook::{
        endpoint::{WebhookEndpoint, WebhookEndpointCreateRequest},
        event_types::WebhookEvent,
    },
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

    // Get webhook URL from environment variable or command line argument
    let webhook_url = env::args()
        .nth(1)
        .or_else(|| env::var("WEBHOOK_URL").ok())
        .expect("Please provide a webhook URL as argument or set WEBHOOK_URL environment variable");

    // Create a new webhook endpoint
    let request = WebhookEndpointCreateRequest::new(webhook_url).with_subscriptions(vec![
        WebhookEvent::LicenseCreated,
        WebhookEvent::LicenseValidationSucceeded,
        WebhookEvent::LicenseValidationFailed,
        WebhookEvent::MachineCreated,
        WebhookEvent::MachineDeleted,
    ]);

    match WebhookEndpoint::create(request).await {
        Ok(endpoint) => {
            println!("Webhook endpoint created: {}", endpoint.id);
            println!("  URL: {}", endpoint.url);
            println!("  Subscriptions: {:?}", endpoint.subscriptions);
            println!("  Signature Algorithm: {:?}", endpoint.signature_algorithm);
            println!("  Created: {}", endpoint.created);
            println!("  Environment ID: {:?}", endpoint.environment_id);
        }
        Err(e) => {
            println!("Failed to create webhook endpoint: {e:?}");
        }
    }

    Ok(())
}
