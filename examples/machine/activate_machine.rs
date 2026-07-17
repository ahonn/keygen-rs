use dotenv::dotenv;
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::LicenseValidationRequest,
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
    })?;

    let fingerprint = machine_uid::get().unwrap_or("".into());
    let validation = keygen_rs::validate(&LicenseValidationRequest::for_fingerprint(
        fingerprint.clone(),
    ))
    .await?;
    if validation.meta.valid {
        println!("License validated successfully");
    } else if let Some(license) = validation.license {
        if matches!(
            validation.meta.code.as_str(),
            "NO_MACHINE" | "NO_MACHINES" | "FINGERPRINT_SCOPE_MISMATCH"
        ) {
            let machine = license.activate(&fingerprint, &[]).await?;
            println!("License activated successfully: {machine:?}");
        } else {
            println!("License validation failed: {}", validation.meta.detail);
        }
    } else {
        println!("License validation failed: {}", validation.meta.detail);
    }

    Ok(())
}
