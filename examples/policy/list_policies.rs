use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    policy::Policy,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    // List all policies
    match Policy::list(None).await {
        Ok(policies) => {
            println!("Found {} policies:", policies.len());
            for policy in policies {
                println!("  ID: {}", policy.id);
                println!("  Name: {}", policy.name);
                println!("  Duration: {:?} seconds", policy.duration);
                println!("  Max Machines: {:?}", policy.max_machines);
                println!("  Expiration Strategy: {:?}", policy.expiration_strategy);
                println!("  Created: {}", policy.created);
                println!("  Relationships:");
                println!("    Account ID: {:?}", policy.account_id);
                println!("    Product ID: {:?}", policy.product_id);
                println!("  ---");
            }
        }
        Err(e) => {
            println!("Failed to list policies: {e:?}");
        }
    }

    Ok(())
}
