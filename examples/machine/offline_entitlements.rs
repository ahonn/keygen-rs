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
    })?;
    
    let config = config::get_config().expect("Failed to get config");
    let license_key = config.license_key.clone().expect("License key required");

    let fingerprint = machine_uid::get().unwrap_or("".into());
    let license = match keygen_rs::validate(&[fingerprint.clone()], &[]).await {
        Ok(license) => license,
        Err(Error::LicenseNotActivated { license, .. }) => license,
        Err(e) => return Err(e),
    };

    // Get the machine for checkout
    let machine = license.machine(&fingerprint).await?;

    // Compare online vs offline entitlements access
    let online_entitlements = license.entitlements(None).await?;
    println!("Online entitlements: {}", online_entitlements.len());

    // Checkout machine file with included relationships
    let options = MachineCheckoutOpts::with_include(vec![
        "license.entitlements".to_string(),
        "components".to_string(),
    ]);
    let machine_file = machine.checkout(&options).await?;
    
    // Decrypt key for machine files is license_key + fingerprint
    let decryption_key = format!("{}{}", license_key, fingerprint);
    
    let offline_entitlements = machine_file.entitlements(&decryption_key)?;
    println!("Offline entitlements: {}", offline_entitlements.len());

    // Pure offline workflow
    let dataset = machine_file.decrypt(&decryption_key)?;
    
    if let Some(entitlements) = dataset.offline_entitlements() {
        println!("Available entitlements:");
        for ent in entitlements {
            println!("  - {} ({})", ent.code, 
                ent.name.as_deref().unwrap_or("No name"));
        }
        
        // Feature checking
        let required_features = vec!["premium", "advanced", "api-access"];
        for feature in required_features {
            let has_feature = entitlements.iter().any(|ent| ent.code == feature);
            println!("Feature '{}': {}", feature, if has_feature { "enabled" } else { "disabled" });
        }
    }

    // Show components if available
    if let Some(components) = dataset.offline_components() {
        println!("Available components:");
        for comp in components {
            println!("  - {} ({})", comp.name, comp.fingerprint);
        }
    }
    
    Ok(())
}