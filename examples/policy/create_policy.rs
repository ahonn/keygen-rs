use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    policy::{AuthenticationStrategy, CreatePolicyRequest, Policy},
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

    // Get product ID from environment variable
    let product_id = env::var("KEYGEN_PRODUCT_ID")
        .expect("KEYGEN_PRODUCT_ID must be set (get from list_products example)");

    // Create a new policy with License authentication strategy
    let request = CreatePolicyRequest {
        name: "License Auth Policy".to_string(),
        product_id,
        authentication_strategy: Some(AuthenticationStrategy::License),
        ..Default::default()
    };

    match Policy::create(request).await {
        Ok(policy) => {
            println!("Policy created: {}", policy.id);
            println!("ID: {}", policy.id);
            println!("Name: {}", policy.name);
            println!("Duration: {:?} seconds", policy.duration);
            println!("Max Machines: {:?}", policy.max_machines);
            println!("Expiration Strategy: {:?}", policy.expiration_strategy);
        }
        Err(e) => {
            println!("Failed to create policy: {:?}", e);
        }
    }

    Ok(())
}
