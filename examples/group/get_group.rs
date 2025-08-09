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

    match Group::get(&group_id).await {
        Ok(group) => {
            println!("Group retrieved successfully!");
            println!("ID: {}", group.id);
            println!("Name: {}", group.name);
            println!("Max Users: {:?}", group.max_users);
            println!("Max Licenses: {:?}", group.max_licenses);
            println!("Max Machines: {:?}", group.max_machines);
            println!("Account ID: {:?}", group.account_id);
            println!("Owner ID: {:?}", group.owner_id);
            println!("Created: {}", group.created.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("Updated: {}", group.updated.format("%Y-%m-%d %H:%M:%S UTC"));

            if let Some(metadata) = &group.metadata {
                println!("Metadata:");
                for (key, value) in metadata {
                    println!("  {}: {}", key, value);
                }
            } else {
                println!("Metadata: None");
            }
        }
        Err(e) => {
            println!("Failed to retrieve group: {e:?}");
            println!("Make sure KEYGEN_GROUP_ID is set to a valid group ID");
        }
    }

    Ok(())
}
