use keygen_rs::{
    config::{self, KeygenConfig},
    user,
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

    // Get user ID and group ID from command line arguments
    let user_id = env::args().nth(1).expect("Usage: cargo run --example change_group <user_id> [group_id]");
    let group_id = env::args().nth(2);

    // Change user group
    match user::change_group(&user_id, group_id.as_deref()).await {
        Ok(user) => {
            println!("✅ User group changed successfully!");
            println!("User ID: {}", user.id);
            println!("Email: {}", user.email);
            if let Some(group_id) = group_id {
                println!("New Group ID: {}", group_id);
            } else {
                println!("Group: Removed from group");
            }
        },
        Err(e) => {
            println!("❌ Failed to change user group: {:?}", e);
        }
    }

    Ok(())
}