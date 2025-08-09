use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    group::Group,
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

    // Get the group ID from environment variable
    let group_id = env::var("KEYGEN_GROUP_ID").expect("KEYGEN_GROUP_ID must be set");

    // First, retrieve the group to show what we're about to delete
    let group = match Group::get(&group_id).await {
        Ok(group) => {
            println!("Group to be deleted:");
            println!("  ID: {}", group.id);
            println!("  Name: {}", group.name);
            println!("  Max Users: {:?}", group.max_users);
            println!("  Max Licenses: {:?}", group.max_licenses);
            println!("  Max Machines: {:?}", group.max_machines);
            println!(
                "  Created: {}",
                group.created.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!();
            group
        }
        Err(e) => {
            println!("Failed to retrieve group: {e:?}");
            println!("Make sure KEYGEN_GROUP_ID is set to a valid group ID");
            return Ok(());
        }
    };

    // Show warning about deletion
    println!("⚠️  WARNING: This will permanently delete the group!");
    println!("   Group Name: {}", group.name);
    println!("   Group ID: {}", group.id);
    println!();

    // Check for confirmation environment variable
    let confirm = env::var("CONFIRM_DELETE").unwrap_or_else(|_| "false".to_string());
    if confirm.to_lowercase() != "true" {
        println!("🛑 Deletion aborted!");
        println!("   To confirm deletion, set the environment variable:");
        println!("   CONFIRM_DELETE=true");
        return Ok(());
    }

    println!("🗑️  Proceeding with deletion...");

    match group.delete().await {
        Ok(()) => {
            println!("✅ Group deleted successfully!");
            println!("   Deleted group: {} ({})", group.name, group.id);
        }
        Err(e) => {
            println!("❌ Failed to delete group: {e:?}");
            println!("   This might happen if:");
            println!("   - The group has associated resources (users, licenses, machines)");
            println!("   - You don't have permission to delete this group");
            println!("   - The group has already been deleted");
        }
    }

    Ok(())
}
