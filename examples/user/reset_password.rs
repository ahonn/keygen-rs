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

    // Get user ID and new password from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: cargo run --example reset_password <user_id> <new_password>");
        return Ok(());
    }
    
    let user_id = &args[1];
    let new_password = &args[2];

    // Reset user password
    match user::reset_password(user_id, new_password).await {
        Ok(()) => {
            println!("✅ Password reset successfully!");
        },
        Err(e) => {
            println!("❌ Failed to reset password: {:?}", e);
        }
    }

    Ok(())
}