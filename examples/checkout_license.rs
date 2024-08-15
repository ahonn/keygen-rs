use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error, license::LicenseCheckoutOpts,
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
    if let Ok(license) = keygen_rs::validate(&[fingerprint]).await {
      let options = LicenseCheckoutOpts {
        ttl: Some(chrono::Duration::days(7)),
        include: None,
      };
      let license_file = license.checkout(&options).await?;
      let dataset = license_file.decrypt(&config.license_key.unwrap())?;
      println!("License checkout successful: {:?}", dataset);
      let _ = license_file.verify()?;
      println!("License file verification successful");
    } else {
        println!("License validation failed");
    };

    Ok(())
}
