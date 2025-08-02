use keygen_rs::{
    config::{self, KeygenConfig},
    entitlement::Entitlement,
    errors::Error,
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

    let args: Vec<String> = env::args().collect();
    let entitlement_id = if args.len() > 1 {
        args[1].clone()
    } else {
        env::var("KEYGEN_ENTITLEMENT_ID").unwrap_or_else(|_| {
            println!("Usage: cargo run --example get_entitlement <entitlement_id>");
            std::process::exit(1);
        })
    };

    match Entitlement::get(&entitlement_id).await {
        Ok(entitlement) => {
            println!("Entitlement: {} ({})", entitlement.code, entitlement.name.unwrap_or_else(|| "No name".to_string()));
        }
        Err(e) => {
            println!("Failed to get entitlement: {:?}", e);
        }
    }

    Ok(())
}