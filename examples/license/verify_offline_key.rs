use base64::{engine::general_purpose, Engine as _};
use dotenv::dotenv;
use ed25519_dalek::{Signer, SigningKey};
use keygen_rs::{
    config::{self, KeygenConfig},
    license::SchemeCode,
};
use rand::rngs::OsRng;
use serde_json::json;
use std::env;

fn generate_signed_license_key(key: String) -> (String, String) {
    let mut csprng = OsRng;
    let keypair: SigningKey = SigningKey::generate(&mut csprng);

    let payload = json!({
        "lic": key,
        "exp": "2025-12-31",
        "iss": "keygen",
    });

    let payload_encoded = general_purpose::URL_SAFE.encode(payload.to_string());
    let signing_input = format!("key/{payload_encoded}");
    let signature = keypair.sign(signing_input.as_bytes());

    let signed_key = format!(
        "{}.{}",
        signing_input,
        general_purpose::URL_SAFE.encode(signature.to_bytes())
    );
    let public_key = hex::encode(keypair.verifying_key().as_bytes());
    (public_key, signed_key)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let (public_key, signed_key) =
        generate_signed_license_key("4F5D3B-0FB8B2-6871BC-5D3EB3-4885B7-V3".to_string());
    config::set_config(KeygenConfig {
        api_url: env::var("KEYGEN_API_URL").expect("KEYGEN_API_URL must be set"),
        account: env::var("KEYGEN_ACCOUNT").expect("KEYGEN_ACCOUNT must be set"),
        product: env::var("KEYGEN_PRODUCT").expect("KEYGEN_PRODUCT must be set"),
        license_key: Some(env::var("KEYGEN_LICENSE_KEY").expect("KEYGEN_LICENSE_KEY must be set")),
        public_key: Some(public_key.clone()),
        ..KeygenConfig::default()
    })
    .expect("Failed to set config");

    println!("Signed key: {signed_key:?}");
    if let Ok(data) = keygen_rs::verify(SchemeCode::Ed25519Sign, &signed_key) {
        println!(
            "License verified: {data:?}",
            data = String::from_utf8_lossy(&data)
        );
    } else {
        println!("License verification failed");
    }
}
