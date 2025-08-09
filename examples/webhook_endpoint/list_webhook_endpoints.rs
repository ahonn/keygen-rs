use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    webhook::endpoint::{WebhookEndpoint, WebhookEndpointListOptions},
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

    // List all webhook endpoints
    let options = WebhookEndpointListOptions {
        limit: Some(10),
        page_size: Some(10),
        page_number: Some(1),
    };

    match WebhookEndpoint::list(Some(&options)).await {
        Ok(endpoints) => {
            println!("Found {} webhook endpoints:", endpoints.len());
            for endpoint in endpoints {
                println!("  ID: {}", endpoint.id);
                println!("  URL: {}", endpoint.url);
                println!("  Subscriptions: {:?}", endpoint.subscriptions);
                println!("  Signature Algorithm: {:?}", endpoint.signature_algorithm);
                println!("  Created: {}", endpoint.created);
                println!("  Updated: {}", endpoint.updated);
                println!("  Environment ID: {:?}", endpoint.environment_id);
                println!("  ---");
            }
        }
        Err(e) => {
            println!("Failed to list webhook endpoints: {e:?}");
        }
    }

    Ok(())
}
