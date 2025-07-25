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

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let auto_confirm = args.contains(&"--yes".to_string());
    
    // Get policy ID from command line argument or environment variable
    let policy_id = args
        .iter()
        .find(|arg| !arg.starts_with("--") && !arg.contains("delete_policy"))
        .cloned()
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
            let should_delete = if auto_confirm {
                println!("🔥 Deleting policy automatically (--yes flag provided)...");
                true
            } else {
                println!("\n⚠️  Are you sure you want to delete this policy?");
                println!("This action cannot be undone and may affect associated licenses.");
                println!("Type 'yes' to confirm deletion (or use --yes flag): ");

                use std::io::{self, BufRead};
                let stdin = io::stdin();
                let mut lines = stdin.lock().lines();

                if let Some(Ok(line)) = lines.next() {
                    line.trim().to_lowercase() == "yes"
                } else {
                    false
                }
            };

            if should_delete {
                // Delete the policy
                match policy.delete().await {
                    Ok(_) => {
                        println!("✅ Policy deleted successfully!");
                    }
                    Err(e) => {
                        println!("❌ Failed to delete policy: {:?}", e);
                    }
                }
            } else {
                println!("❌ Deletion cancelled.");
            }
        }
        Err(e) => {
            println!("❌ Failed to get policy: {:?}", e);
        }
    }

    Ok(())
}
