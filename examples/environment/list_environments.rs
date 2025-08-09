use keygen_rs::{
    config::{self, KeygenConfig},
    environment::{Environment, ListEnvironmentsOptions},
    errors::Error,
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

    // Set up pagination options
    let options = ListEnvironmentsOptions {
        limit: Some(10),
        page_size: Some(5),
        page_number: Some(1),
    };

    // List environments with pagination
    match Environment::list(Some(options)).await {
        Ok(result) => {
            println!("Found {} environments:", result.environments.len());
            for environment in result.environments {
                println!("  ID: {}", environment.id);
                println!("  Name: {}", environment.name);
                println!("  Code: {}", environment.code);
                println!("  Isolation Strategy: {:?}", environment.isolation_strategy);
                println!("  Created: {}", environment.created);
                println!("  Updated: {}", environment.updated);
                println!("  Account ID: {:?}", environment.account_id);
                println!("  ---");
            }

            if let Some(meta) = result.meta {
                println!("Pagination metadata: {}", meta);
            }

            if let Some(links) = result.links {
                println!("Pagination links: {}", links);
            }
        }
        Err(e) => {
            println!("Failed to list environments: {e:?}");
        }
    }

    Ok(())
}
