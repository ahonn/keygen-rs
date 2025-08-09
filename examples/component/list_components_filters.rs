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

    // Example 1: Basic listing with pagination
    println!("Basic listing (limit: 5)");
    let basic_options = ListComponentsOptions::new()
        .with_limit(5)
        .with_pagination(1, 5);

    match Component::list(Some(basic_options)).await {
        Ok(components) => {
            println!("Found {} component(s)", components.len());
            for component in &components {
                println!("{}: {}", component.id, component.name);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Example 2: Filter by machine
    if let Ok(machine_id) = env::var("KEYGEN_MACHINE_ID") {
        println!("\nFilter by machine: {}", machine_id);
        let machine_options = ListComponentsOptions::new().with_machine(machine_id);
        match Component::list(Some(machine_options)).await {
            Ok(components) => {
                println!("Found {} component(s)", components.len());
                for component in &components {
                    println!("{}: {}", component.id, component.name);
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    // Filter by license
    if let Ok(license_id) = env::var("KEYGEN_LICENSE_ID") {
        println!("\nFilter by license: {}", license_id);
        let license_options = ListComponentsOptions::new().with_license(license_id);
        match Component::list(Some(license_options)).await {
            Ok(components) => {
                println!("Found {} component(s)", components.len());
                for component in &components {
                    println!("{}: {}", component.id, component.name);
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    // Filter by product
    if let Ok(product_id) = env::var("KEYGEN_PRODUCT_ID") {
        println!("\nFilter by product: {}", product_id);
        let product_options = ListComponentsOptions::new().with_product(product_id);
        match Component::list(Some(product_options)).await {
            Ok(components) => {
                println!("Found {} component(s)", components.len());
                for component in &components {
                    println!("{}: {}", component.id, component.name);
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }

    // Combined filters
    let mut options = ListComponentsOptions::new();
    if let Ok(machine_id) = env::var("KEYGEN_MACHINE_ID") {
        options = options.with_machine(machine_id);
    }
    if let Ok(license_id) = env::var("KEYGEN_LICENSE_ID") {
        options = options.with_license(license_id);
    }

    println!("\nCombined filters");
    match Component::list(Some(options)).await {
        Ok(components) => {
            println!(
                "Found {} component(s) with combined filters",
                components.len()
            );
            for component in components {
                println!("{}: {}", component.id, component.name);
            }
        }
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
