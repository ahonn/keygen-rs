use keygen_rs::{
    config::{self, KeygenConfig},
    entitlement::Entitlement,
    errors::Error,
};
use std::{
    env,
    io::{self, Write},
};

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

    let args: Vec<String> = env::args().collect();
    let entitlement_id = if args.len() > 1 {
        args[1].clone()
    } else {
        env::var("KEYGEN_ENTITLEMENT_ID").unwrap_or_else(|_| {
            println!("Usage: cargo run --example delete_entitlement <entitlement_id>");
            std::process::exit(1);
        })
    };

    let entitlement = Entitlement::get(&entitlement_id).await?;

    print!("Delete entitlement '{}'? (yes/no): ", entitlement.code);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().to_lowercase() == "yes" {
        match entitlement.delete().await {
            Ok(()) => println!("Entitlement deleted: {}", entitlement.code),
            Err(e) => println!("Failed to delete entitlement: {e:?}"),
        }
    } else {
        println!("Deletion cancelled");
    }

    Ok(())
}
