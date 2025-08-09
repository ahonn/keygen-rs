use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    webhook::{
        endpoint::{WebhookEndpoint, WebhookEndpointUpdateRequest},
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

    // Get webhook endpoint ID from command line arguments or environment
    let endpoint_id = env::args()
        .nth(1)
        .or_else(|| env::var("WEBHOOK_ENDPOINT_ID").ok())
        .expect("Please provide a webhook endpoint ID as an argument or set WEBHOOK_ENDPOINT_ID");

    // First, get the existing webhook endpoint
    match WebhookEndpoint::get(&endpoint_id).await {
        Ok(endpoint) => {
            println!("Found webhook endpoint:");
            println!("  Current URL: {}", endpoint.url);
            println!("  Current Subscriptions: {:?}", endpoint.subscriptions);
            println!(
                "  Current Signature Algorithm: {:?}",
                endpoint.signature_algorithm
            );

            // Update the webhook endpoint
            let update_request = WebhookEndpointUpdateRequest::new().with_subscriptions(vec![
                WebhookEvent::LicenseCreated,
                WebhookEvent::LicenseUpdated,
                WebhookEvent::LicenseDeleted,
                WebhookEvent::LicenseExpired,
                WebhookEvent::LicenseRenewed,
                WebhookEvent::MachineCreated,
                WebhookEvent::MachineDeleted,
                WebhookEvent::MachineHeartbeatPing,
                WebhookEvent::UserCreated,
                WebhookEvent::UserDeleted,
            ]);

            match endpoint.update(update_request).await {
                Ok(updated_endpoint) => {
                    println!("Webhook endpoint updated: {}", updated_endpoint.id);
                    println!("  New URL: {}", updated_endpoint.url);
                    println!("  New Subscriptions: {:?}", updated_endpoint.subscriptions);
                    println!(
                        "  New Signature Algorithm: {:?}",
                        updated_endpoint.signature_algorithm
                    );
                    println!("  Updated: {}", updated_endpoint.updated);
                }
                Err(e) => {
                    println!("Failed to update webhook endpoint: {e:?}");
                }
            }
        }
        Err(e) => {
            println!("Failed to find webhook endpoint: {e:?}");
        }
    }

    Ok(())
}
