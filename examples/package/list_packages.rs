use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    package::{ListPackagesOptions, Package},
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();

    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").unwrap_or_else(|_| "https://api.keygen.sh".to_string()),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        token: Some(env::var("KEYGEN_ADMIN_TOKEN").expect("KEYGEN_ADMIN_TOKEN must be set")),
        ..KeygenConfig::default()
    })?;

    let options = ListPackagesOptions {
        limit: Some(10),
        ..Default::default()
    };

    match Package::list(Some(options)).await {
        Ok(packages) => {
            println!("Found {} packages:", packages.len());
            for package in packages {
                println!(
                    "  - {} ({}) [ID: {}]",
                    package.name, package.key, package.id
                );
                if let Some(engine) = &package.engine {
                    println!("    Engine: {:?}", engine);
                }
            }
        }
        Err(e) => {
            println!("Failed to list packages: {e:?}");
        }
    }

    Ok(())
}
