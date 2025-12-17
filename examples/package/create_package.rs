use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    package::{CreatePackageRequest, Package, PackageEngine},
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

    let product_id = env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set");

    let request = CreatePackageRequest {
        name: "My Application".to_string(),
        key: format!("my-app-{}", chrono::Utc::now().timestamp() % 1000),
        product_id,
        engine: Some(PackageEngine::Raw),
        metadata: None,
    };

    match Package::create(request).await {
        Ok(package) => {
            println!("Package created successfully!");
            println!("  ID: {}", package.id);
            println!("  Name: {}", package.name);
            println!("  Key: {}", package.key);
            println!("  Engine: {:?}", package.engine);
            println!("  Created: {}", package.created);
        }
        Err(e) => {
            println!("Failed to create package: {e:?}");
        }
    }

    Ok(())
}
