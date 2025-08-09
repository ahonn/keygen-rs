use dotenv::dotenv;
use keygen_rs::{
    component::{Component, ListComponentsOptions},
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

    // Create list options with various filters
    let options = ListComponentsOptions::new()
        .with_limit(10)
        .with_pagination(1, 10);

    // Add optional filters if environment variables are set
    let options = if let Ok(machine_id) = env::var("KEYGEN_MACHINE_ID") {
        options.with_machine(machine_id)
    } else {
        options
    };

    let options = if let Ok(license_id) = env::var("KEYGEN_LICENSE_ID") {
        options.with_license(license_id)
    } else {
        options
    };

    let options = if let Ok(product_id) = env::var("KEYGEN_PRODUCT_ID") {
        options.with_product(product_id)
    } else {
        options
    };

    match Component::list(Some(options)).await {
        Ok(components) => {
            println!("Found {} component(s)", components.len());
            for component in components {
                println!("ID: {}", component.id);
                println!("Name: {}", component.name);
                println!("Fingerprint: {}", component.fingerprint);
                println!("Machine ID: {:?}", component.machine_id);
                println!("License ID: {:?}", component.license_id);
                println!("Created: {}", component.created);
                println!();
            }
        }
        Err(e) => {
            println!("Failed to list components: {e:?}");
        }
    }

    Ok(())
}
