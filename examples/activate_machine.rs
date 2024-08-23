use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        public_key: Some(env::var("KEYGEN_PUBLIC_KEY").expect("KEYGEN_PUBLIC_KEY must be set")),
        ..KeygenConfig::default()
    });

    let fingerprint = machine_uid::get().unwrap_or("".into());
    if let Err(err) = keygen_rs::validate(&[fingerprint.clone()]).await {
        match err {
            Error::LicenseNotActivated { license, .. } => {
                let machine = license.activate(&fingerprint, &[]).await?;
                println!("License activated successfully: {:?}", machine);
            }
            _ => {
                println!("License validation failed: {:?}", err);
            }
        }
    } else {
        println!("License validated successfully");
    }

    Ok(())
}
