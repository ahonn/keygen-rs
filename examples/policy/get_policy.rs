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
    });

    // Get policy ID from command line argument or environment variable
    let policy_id = env::args()
        .nth(1)
        .or_else(|| env::var("POLICY_ID").ok())
        .expect("Please provide a policy ID as argument or set POLICY_ID environment variable");

    // Get the policy
    match Policy::get(&policy_id).await {
        Ok(policy) => {
            println!("✅ Policy found!");
            println!("  ID: {}", policy.id);
            println!("  Name: {}", policy.name);
            println!("  Duration: {:?} seconds", policy.duration);
            println!(
                "  Authentication Strategy: {:?}",
                policy.authentication_strategy
            );
            println!("  Expiration Strategy: {:?}", policy.expiration_strategy);
            println!(
                "  Machine Leasing Strategy: {:?}",
                policy.machine_leasing_strategy
            );
            println!("  Max Machines: {:?}", policy.max_machines);
            println!("  Max Processes: {:?}", policy.max_processes);
            println!("  Max Uses: {:?}", policy.max_uses);
            println!("  Max Cores: {:?}", policy.max_cores);
            println!("  Floating: {}", policy.floating);
            println!("  Strict: {}", policy.strict);
            println!("  Encrypted: {}", policy.encrypted);
            println!("  Protected: {}", policy.protected);
            println!("  Require Heartbeat: {}", policy.require_heartbeat);
            if policy.require_heartbeat {
                println!("  Heartbeat Duration: {:?}", policy.heartbeat_duration);
                println!(
                    "  Heartbeat Cull Strategy: {:?}",
                    policy.heartbeat_cull_strategy
                );
            }
            println!("  Require Check-in: {}", policy.require_check_in);
            if policy.require_check_in {
                println!("  Check-in Interval: {:?}", policy.check_in_interval);
                println!(
                    "  Check-in Interval Count: {:?}",
                    policy.check_in_interval_count
                );
            }
            println!("  Metadata: {:?}", policy.metadata);
            println!("  Created: {}", policy.created);
            println!("  Updated: {}", policy.updated);
        }
        Err(e) => {
            println!("❌ Failed to get policy: {:?}", e);
        }
    }

    Ok(())
}
