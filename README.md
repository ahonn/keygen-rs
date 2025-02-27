# Unofficial Keygen Rust SDK

[![Crates.io](https://img.shields.io/crates/v/keygen-rs.svg)](https://crates.io/crates/keygen-rs)
[![Documentation](https://docs.rs/keygen-rs/badge.svg)](https://docs.rs/keygen-rs)

The `keygen-rs` crate allows Rust programs to license using the [keygen.sh](https://keygen.sh) service.

### Sponsored by

<a href="https://keygen.sh?via=keygen-rs" style="margin-right: 10px">
    <img src="https://keygen.sh/images/logo-pill.png" width="200" alt="Keygen">
</a>
<a href="https://badgeify.app?ref=keygen-rs">
    <img src="https://badgeify.app/logo-pill.png" width="200" alt="Badgeify">
</a>

## Installing

Add this to your `Cargo.toml`:

```toml
[dependencies]
keygen-rs = "0.3.1"
```

### Tauri Plugin

A Tauri plugin for this SDK is available as `tauri-plugin-keygen-rs`.
It provides an easy way to integrate Keygen licensing into your Tauri applications. For more information, check [the plugin's README](./packages/tauri-plugin-keygen-rs/README.md).

## Config

### KeygenConfig

Use `KeygenConfig` to configure the SDK globally. You should set this before making any API calls.

```rust
use keygen_rs::config::{self, KeygenConfig};

config::set_config(KeygenConfig {
    api_url: "https://api.keygen.sh".to_string(),
    account: "YOUR_KEYGEN_ACCOUNT_ID".to_string(),
    product: "YOUR_KEYGEN_PRODUCT_ID".to_string(),
    license_key: Some("A_KEYGEN_LICENSE_KEY".to_string()),
    public_key: Some("YOUR_KEYGEN_PUBLIC_KEY".to_string()),
    ..KeygenConfig::default()
});
```

## Usage

### Validate a License

To validate a license, configure `KeygenConfig` with your Keygen account details. Then call the `validate` function with a device fingerprint:

```rust
use keygen_rs::{config::{self, KeygenConfig}, errors::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    config::set_config(KeygenConfig {
        api_url: "https://api.keygen.sh".to_string(),
        account: "YOUR_KEYGEN_ACCOUNT_ID".to_string(),
        product: "YOUR_KEYGEN_PRODUCT_ID".to_string(),
        license_key: Some("A_KEYGEN_LICENSE_KEY".to_string()),
        public_key: Some("YOUR_KEYGEN_PUBLIC_KEY".to_string()),
        ..KeygenConfig::default()
    });

    let fingerprint = machine_uid::get().unwrap_or("".into());
    let license = keygen_rs::validate(&[fingerprint]).await?;
    println!("License validated successfully: {:?}", license);

    Ok(())
}
```

### Activate a Machine

To activate a machine for a license:

```rust
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    config::set_config(KeygenConfig {
        // ... (configuration)
    });

    let fingerprint = machine_uid::get().unwrap_or("".into());
    if let Err(err) = keygen_rs::validate(&[fingerprint.clone()], &[]).await {
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
```

### Offline License Key Verification

To verify a signed license key offline:

```rust
use keygen_rs::{config::{self, KeygenConfig}, license::SchemeCode};

fn main() {
    config::set_config(KeygenConfig {
        // ... (configuration)
    });

    let signed_key = "YOUR_SIGNED_LICENSE_KEY";
    if let Ok(data) = keygen_rs::verify(SchemeCode::Ed25519Sign, signed_key) {
        println!("License verified: {:?}", String::from_utf8_lossy(&data));
    } else {
        println!("License verification failed");
    }
}
```

## Error Handling

The SDK returns meaningful errors which can be handled in your integration. Here's an example of handling a `LicenseNotActivated` error:

```rust
use keygen_rs::{config::{self, KeygenConfig}, errors::Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    config::set_config(KeygenConfig {
        // ... (configuration)
    });

    let fingerprint = machine_uid::get().unwrap_or("".into());
    match keygen_rs::validate(&[fingerprint.clone()]).await {
        Ok(license) => println!("License is valid: {:?}", license),
        Err(Error::LicenseNotActivated { license, .. }) => {
            println!("License is not activated. Activating...");
            let machine = license.activate(&fingerprint, &[]).await?;
            println!("Machine activated: {:?}", machine);
        },
        Err(e) => println!("Error: {:?}", e),
    }

    Ok(())
}
```

## Examples

For more detailed examples, please refer to the `examples` directory in the repository.

## Testing

When implementing a testing strategy for your licensing integration, we recommend mocking the Keygen API responses. This is especially important for CI/CD environments to prevent unnecessary load on Keygen's servers and to stay within your account's daily request limits.
You can use crates like `mockito` or `wiremock` to mock HTTP responses in your tests.

## Inspired by

- [keygen-go](https://github.com/keygen-sh/keygen-go)
- [tauri-plugin-keygen](https://github.com/bagindo/tauri-plugin-keygen)

## License

This project is licensed under the MIT License.
