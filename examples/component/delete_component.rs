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

    // First, retrieve the component to show what we're deleting
    let component = match Component::get(&component_id).await {
        Ok(component) => component,
        Err(e) => {
            println!("Failed to retrieve component: {e:?}");
            return Ok(());
        }
    };

    match component.delete().await {
        Ok(()) => {
            println!("Component deleted: {}", component.id);
        }
        Err(e) => {
            println!("Failed to delete component: {e:?}");
        }
    }

    Ok(())
}
