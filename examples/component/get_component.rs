use dotenv::dotenv;
use keygen_rs::{
    component::Component,
    config::{self, KeygenConfig},
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load environment variables from .env file
    dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // Get the component ID from environment variable
    let component_id = env::var("KEYGEN_COMPONENT_ID").expect("KEYGEN_COMPONENT_ID must be set");

    match Component::get(&component_id).await {
        Ok(component) => {
            println!("ID: {}", component.id);
            println!("Fingerprint: {}", component.fingerprint);
            println!("Name: {}", component.name);
            println!("Account ID: {:?}", component.account_id);
            println!("Environment ID: {:?}", component.environment_id);
            println!("Product ID: {:?}", component.product_id);
            println!("License ID: {:?}", component.license_id);
            println!("Machine ID: {:?}", component.machine_id);
            println!("Metadata: {:?}", component.metadata);
            println!("Created: {}", component.created);
            println!("Updated: {}", component.updated);
        }
        Err(e) => {
            println!("Failed to retrieve component: {e:?}");
        }
    }

    Ok(())
}
