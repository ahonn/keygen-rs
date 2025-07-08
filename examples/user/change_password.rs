use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    user,
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

    // Get user ID and passwords from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage: cargo run --example change_password <user_id> <current_password> <new_password>");
        return Ok(());
    }

    let user_id = &args[1];
    let current_password = &args[2];
    let new_password = &args[3];

    // Change user password
    match user::change_password(user_id, current_password, new_password).await {
        Ok(()) => {
            println!("✅ Password changed successfully!");
        }
        Err(e) => {
            println!("❌ Failed to change password: {:?}", e);
        }
    }

    Ok(())
}
