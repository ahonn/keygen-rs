use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    machine::MachineCheckoutOpts,
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
    let config = config::get_config();

    let fingerprint = machine_uid::get().unwrap_or("".into());
    if let Ok(license) = keygen_rs::validate(&[fingerprint.clone()], &[]).await {
        let machine = license.machine(&fingerprint).await?;
        let options = MachineCheckoutOpts {
            ttl: Some(604800), // 7 days in seconds
            include: None,
        };
        let machine_file = machine.checkout(&options).await?;
        if machine_file.verify().is_ok() {
            // the encryption secret for a machine file is the license key concatenated with the machine fingerprint
            // https://keygen.sh/docs/api/cryptography/#cryptographic-lic-decrypt
            let key = format!("{}{}", config.license_key.unwrap(), machine.fingerprint);
            let dataset = machine_file.decrypt(&key)?;
            println!("Machine checkout successful: {:?}", dataset);
        }
    } else {
        println!("License validation failed");
    };

    Ok(())
}
