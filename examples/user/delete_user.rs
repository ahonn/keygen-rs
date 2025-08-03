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
    })?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let auto_confirm = args.contains(&"--yes".to_string());

    // Get user ID from command line argument
    let user_id = args
        .iter()
        .find(|arg| !arg.starts_with("--") && !arg.contains("delete_user"))
        .cloned()
        .expect("Usage: cargo run --example delete_user <user_id> [--yes]");

    // Confirm deletion
    let should_delete = if auto_confirm {
        println!("Deleting user '{user_id}' automatically (--yes flag provided)...");
        true
    } else {
        println!("Are you sure you want to delete user '{user_id}'? This action cannot be undone.");
        println!("Type 'yes' to confirm (or use --yes flag):");

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        input.trim().to_lowercase() == "yes"
    };

    if !should_delete {
        println!("Deletion cancelled.");
        return Ok(());
    }

    // Delete user
    match user::delete(&user_id).await {
        Ok(()) => {
            println!("user action completed");
        }
        Err(e) => {
            println!("Failed to delete user: {e:?}");
        }
    }

    Ok(())
}
