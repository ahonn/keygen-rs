use keygen_rs::{
    config::{self, KeygenConfig},
    policy::Policy,
    errors::Error,
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

    // First, get the policy to confirm it exists
    match Policy::get(&policy_id).await {
        Ok(policy) => {
            println!("📦 Found policy:");
            println!("  ID: {}", policy.id);
            println!("  Name: {}", policy.name);
            println!("  Max Machines: {:?}", policy.max_machines);
            
            // Confirm deletion
            println!("\n⚠️  Are you sure you want to delete this policy?");
            println!("This action cannot be undone and may affect associated licenses.");
            println!("Type 'yes' to confirm deletion: ");
            
            use std::io::{self, BufRead};
            let stdin = io::stdin();
            let mut lines = stdin.lock().lines();
            
            if let Some(Ok(line)) = lines.next() {
                if line.trim().to_lowercase() == "yes" {
                    // Delete the policy
                    match policy.delete().await {
                        Ok(_) => {
                            println!("✅ Policy deleted successfully!");
                        },
                        Err(e) => {
                            println!("❌ Failed to delete policy: {:?}", e);
                        }
                    }
                } else {
                    println!("❌ Deletion cancelled.");
                }
            }
        },
        Err(e) => {
            println!("❌ Failed to get policy: {:?}", e);
        }
    }

    Ok(())
}