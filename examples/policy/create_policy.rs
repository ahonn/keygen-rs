use keygen_rs::{
    config::{self, KeygenConfig},
    policy::{self, CreatePolicyRequest, ExpirationStrategy, AuthenticationStrategy},
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Set up configuration with Admin Token
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    });

    // Create a new policy
    let request = CreatePolicyRequest {
        name: "Standard License".to_string(),
        duration: Some(31536000), // 1 year in seconds
        expiration_strategy: ExpirationStrategy::RestrictAccess,
        authentication_strategy: AuthenticationStrategy::Token,
        machine_uniqueness_strategy: "UNIQUE_PER_ACCOUNT".to_string(),
        machine_matching_strategy: "MATCH_ALL".to_string(),
        component_uniqueness_strategy: "UNIQUE_PER_ACCOUNT".to_string(),
        component_matching_strategy: "MATCH_ALL".to_string(),
        process_uniqueness_strategy: "UNIQUE_PER_MACHINE".to_string(),
        process_matching_strategy: "MATCH_ALL".to_string(),
        max_machines: Some(5),
        max_components: None,
        max_processes: None,
        max_cores: None,
        max_uses: None,
        metadata: None,
    };

    match policy::create(request).await {
        Ok(policy) => {
            println!("✅ Policy created successfully!");
            println!("ID: {}", policy.id);
            println!("Name: {}", policy.name);
            println!("Duration: {:?} seconds", policy.duration);
            println!("Max Machines: {:?}", policy.max_machines);
            println!("Expiration Strategy: {:?}", policy.expiration_strategy);
        },
        Err(e) => {
            println!("❌ Failed to create policy: {:?}", e);
        }
    }

    Ok(())
}