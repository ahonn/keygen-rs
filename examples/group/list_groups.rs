use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    group::{Group, ListGroupsOptions},
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

    // Example 1: List all groups
    println!("=== Listing all groups ===");
    match Group::list(None).await {
        Ok(groups) => {
            println!("Found {} groups:", groups.len());
            for group in groups {
                println!(
                    "  Name: {} | ID: {} | Users: {:?} | Licenses: {:?} | Machines: {:?}",
                    group.name, group.id, group.max_users, group.max_licenses, group.max_machines
                );
            }
        }
        Err(e) => {
            println!("Failed to list groups: {e:?}");
        }
    }

    println!();

    // Example 2: List groups with pagination
    println!("=== Listing groups with pagination (first 5) ===");
    let options = ListGroupsOptions {
        limit: Some(5),
        page_size: Some(5),
        page_number: Some(1),
    };

    match Group::list(Some(options)).await {
        Ok(groups) => {
            println!("Found {} groups (first page):", groups.len());
            for group in groups {
                println!(
                    "  Name: {} | ID: {} | Created: {}",
                    group.name,
                    group.id,
                    group.created.format("%Y-%m-%d %H:%M:%S UTC")
                );
                if let Some(metadata) = &group.metadata {
                    println!("    Metadata: {:?}", metadata);
                }
            }
        }
        Err(e) => {
            println!("Failed to list groups with pagination: {e:?}");
        }
    }

    Ok(())
}
