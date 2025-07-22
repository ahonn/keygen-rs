use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    policy::{CreatePolicyRequest, Policy},
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
    });

    // Get product ID from environment variable
    let product_id = env::var("KEYGEN_PRODUCT_ID")
        .or_else(|_| env::var("PRODUCT_ID"))
        .expect("KEYGEN_PRODUCT_ID or PRODUCT_ID must be set");

    // Create a new policy with only required fields (according to docs: name + product relationship)
    let request = CreatePolicyRequest {
        name: "Basic Policy".to_string(),
        product_id,
        ..Default::default()
    };

    match Policy::create(request).await {
        Ok(policy) => {
            println!("✅ Policy created successfully!");
            println!("ID: {}", policy.id);
            println!("Name: {}", policy.name);
            println!("Duration: {:?} seconds", policy.duration);
            println!("Max Machines: {:?}", policy.max_machines);
            println!("Expiration Strategy: {:?}", policy.expiration_strategy);
        }
        Err(e) => {
            println!("❌ Failed to create policy: {:?}", e);
        }
    }

    Ok(())
}
