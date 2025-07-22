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

    // Get user ID from command line argument
    let user_id = env::args()
        .nth(1)
        .expect("Usage: cargo run --example delete_user <user_id>");

    // Confirm deletion
    println!(
        "⚠️  Are you sure you want to delete user '{}'? This action cannot be undone.",
        user_id
    );
    println!("Type 'yes' to confirm:");

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    if input.trim().to_lowercase() != "yes" {
        println!("❌ Deletion cancelled.");
        return Ok(());
    }

    // Delete user
    match user::delete(&user_id).await {
        Ok(()) => {
            println!("✅ User deleted successfully!");
        }
        Err(e) => {
            println!("❌ Failed to delete user: {:?}", e);
        }
    }

    Ok(())
}
