use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    policy::{AuthenticationStrategy, ExpirationStrategy, Policy, UpdatePolicyRequest},
};
use std::collections::HashMap;
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

    // Get policy ID from command line argument or environment variable
    let policy_id = env::args()
        .nth(1)
        .or_else(|| env::var("POLICY_ID").ok())
        .expect("Please provide a policy ID as argument or set POLICY_ID environment variable");

    // First, get the current policy details
    match Policy::get(&policy_id).await {
        Ok(policy) => {
            println!("📦 Found policy:");
            println!("  Current Name: {}", policy.name);
            println!("  Current Max Machines: {:?}", policy.max_machines);
            println!("  Current Duration: {:?} seconds", policy.duration);
            println!(
                "  Current Expiration Strategy: {:?}",
                policy.expiration_strategy
            );

            // Create update request
            let mut metadata = HashMap::new();
            metadata.insert(
                "updated_by".to_string(),
                serde_json::json!("policy_update_example"),
            );
            metadata.insert(
                "update_reason".to_string(),
                serde_json::json!("Testing policy update functionality"),
            );

            let update_request = UpdatePolicyRequest {
                name: Some(format!("{} (Updated)", policy.name)),
                duration: Some(63072000), // 2 years in seconds
                expiration_strategy: Some(ExpirationStrategy::MaintainAccess), // Change strategy
                authentication_strategy: Some(AuthenticationStrategy::License), // Allow license key auth
                metadata: Some(metadata),
                ..Default::default()
            };

            // Update the policy
            match policy.update(update_request).await {
                Ok(updated_policy) => {
                    println!("\n✅ Policy updated successfully!");
                    println!("  New Name: {}", updated_policy.name);
                    println!("  New Max Machines: {:?}", updated_policy.max_machines);
                    println!("  New Duration: {:?} seconds", updated_policy.duration);
                    println!(
                        "  New Expiration Strategy: {:?}",
                        updated_policy.expiration_strategy
                    );
                    println!(
                        "  New Overage Strategy: {:?}",
                        updated_policy.overage_strategy
                    );
                    println!("  New Floating: {}", updated_policy.floating);
                    println!("  New Metadata: {:?}", updated_policy.metadata);
                }
                Err(e) => {
                    println!("\n❌ Failed to update policy: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to get policy: {:?}", e);
        }
    }

    Ok(())
}
