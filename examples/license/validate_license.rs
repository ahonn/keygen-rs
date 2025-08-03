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
    })
    .expect("Failed to set config");

    let fingerprint = machine_uid::get().unwrap_or("".into());
    let license = keygen_rs::validate(&[fingerprint], &[]).await?;

    println!("License validated: {}", license.id);
    println!("License ID: {}", license.id);
    println!("License Key: {}", license.key);
    println!("Status: {:?}", license.status);
    println!("Uses: {:?}", license.uses);
    println!("Max Machines: {:?}", license.max_machines);
    println!("Max Cores: {:?}", license.max_cores);
    println!("Max Uses: {:?}", license.max_uses);
    println!("Max Processes: {:?}", license.max_processes);
    println!("Protected: {:?}", license.protected);
    println!("Suspended: {:?}", license.suspended);

    Ok(())
}
