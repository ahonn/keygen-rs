use keygen_rs::{
    arch::{Arch, ListArchesOptions},
    config::{self, KeygenConfig},
    errors::Error,
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

    let options = ListArchesOptions {
        limit: Some(25),
        ..Default::default()
    };

    match Arch::list(Some(options)).await {
        Ok(arches) => {
            println!("Found {} architectures:", arches.len());
            for arch in arches {
                let name = arch.name.unwrap_or_else(|| "(unnamed)".to_string());
                println!("  - {} (key: {}) [ID: {}]", name, arch.key, arch.id);
            }
        }
        Err(e) => {
            println!("Failed to list architectures: {e:?}");
        }
    }

    Ok(())
}
