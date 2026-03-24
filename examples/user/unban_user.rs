use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    user::User,
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

    // Get user ID from command line argument
    let user_id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example unban_user <user_id>");

    // Unban user
    let user = User::get(&user_id).await?;
    match user.unban().await {
        Ok(user) => {
            println!("user action completed");
            println!("ID: {}", user.id);
            println!("Email: {}", user.email);
            println!("Status: {:?}", user.status);
        }
        Err(e) => {
            println!("Failed to unban user: {e:?}");
        }
    }

    Ok(())
}
