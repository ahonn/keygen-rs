# Unofficial Keygen Rust SDK

[![Crates.io](https://img.shields.io/crates/v/keygen-rs.svg)](https://crates.io/crates/keygen-rs)
[![Documentation](https://docs.rs/keygen-rs/badge.svg)](https://docs.rs/keygen-rs)

The `keygen-rs` crate is an unofficial Rust SDK for integrating with the [keygen.sh](https://keygen.sh) licensing service.

## Features

- **License Management**: Validate, activate, and verify licenses offline
- **Machine Management**: Activate, deactivate, and manage machines
- **Administrative APIs**: Full CRUD operations for products, policies, licenses, users, and tokens (requires admin token)
- **Offline Verification**: Verify signed license keys without internet connectivity
- **Type Safety**: Strongly-typed enums for all API options
- **Service Introspection**: Check API availability and feature support

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
keygen-rs = "0.4"
```

### Feature Flags

The SDK uses feature flags to minimize binary size:

- **`license-key`** (default): End-user features for license validation and machine activation
- **`token`**: Administrative features requiring token authentication

```toml
# For end-user features only (default)
keygen-rs = "0.4"

# For administrative features
keygen-rs = { version = "0.4", features = ["token"] }

# For both end-user and administrative features
keygen-rs = { version = "0.4", features = ["license-key", "token"] }
```

## Tauri Plugin

Tauri plugins for this SDK are available:

- [tauri-plugin-keygen-rs](./packages/tauri-plugin-keygen-rs) for Tauri v1
- [tauri-plugin-keygen-rs2](./packages/tauri-plugin-keygen-rs2) for Tauri v2

These plugins provide an easy way to integrate Keygen licensing into your Tauri applications. For more information, check the plugins' respective READMEs.

## Config

### KeygenConfig

Use `KeygenConfig` to configure the SDK globally. You should set this before making any API calls.

#### For End Users (License Key Authentication)

```rust
use keygen_rs::config::{self, KeygenConfig};

config::set_config(KeygenConfig::license_key(
    "YOUR_KEYGEN_ACCOUNT_ID",
    "YOUR_KEYGEN_PRODUCT_ID", 
    "A_KEYGEN_LICENSE_KEY",
    Some("YOUR_KEYGEN_PUBLIC_KEY"),
));
```

#### For Administrators (Token Authentication)

```rust
use keygen_rs::config::{self, KeygenConfig};

config::set_config(KeygenConfig::admin(
    "YOUR_KEYGEN_ACCOUNT_ID",
    "YOUR_ADMIN_TOKEN",
));
```

#### Custom Configuration

```rust
use keygen_rs::config::{self, KeygenConfig};

config::set_config(KeygenConfig {
    api_url: "https://api.keygen.sh".to_string(), // or your custom domain
    account: "YOUR_KEYGEN_ACCOUNT_ID".to_string(),
    product: "YOUR_KEYGEN_PRODUCT_ID".to_string(),
    license_key: Some("A_KEYGEN_LICENSE_KEY".to_string()),
    token: Some("YOUR_ADMIN_TOKEN".to_string()),
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
    config::set_config(KeygenConfig::license_key(
        "YOUR_KEYGEN_ACCOUNT_ID",
        "YOUR_KEYGEN_PRODUCT_ID",
        "A_KEYGEN_LICENSE_KEY",
        Some("YOUR_KEYGEN_PUBLIC_KEY"),
    ));

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
    config::set_config(KeygenConfig::license_key(
        "YOUR_KEYGEN_ACCOUNT_ID",
        "YOUR_KEYGEN_PRODUCT_ID",
        "A_KEYGEN_LICENSE_KEY",
        Some("YOUR_KEYGEN_PUBLIC_KEY"),
    ));

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
```

### Offline License Key Verification

To verify a signed license key offline:

```rust
use keygen_rs::{config::{self, KeygenConfig}, license::SchemeCode};

fn main() {
    config::set_config(KeygenConfig::license_key(
        "YOUR_KEYGEN_ACCOUNT_ID",
        "YOUR_KEYGEN_PRODUCT_ID",
        "A_KEYGEN_LICENSE_KEY",
        Some("YOUR_KEYGEN_PUBLIC_KEY"),
    ));

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
    config::set_config(KeygenConfig::license_key(
        "YOUR_KEYGEN_ACCOUNT_ID",
        "YOUR_KEYGEN_PRODUCT_ID",
        "A_KEYGEN_LICENSE_KEY",
        Some("YOUR_KEYGEN_PUBLIC_KEY"),
    ));

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

## Administrative APIs

When configured with a token, you can access administrative features:

### Product Management

```rust
use keygen_rs::product::{Product, CreateProductRequest, DistributionStrategy};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a new product
    let product = Product::create(CreateProductRequest {
        name: "My App".to_string(),
        distribution_strategy: Some(DistributionStrategy::Licensed),
        platforms: Some(vec![Platform::MacOs, Platform::Windows]),
        ..Default::default()
    }).await?;

    // List all products
    let products = Product::list(None).await?;
    
    Ok(())
}
```

### Policy Management

```rust
use keygen_rs::policy::{Policy, CreatePolicyRequest, AuthenticationStrategy};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a new policy
    let policy = Policy::create(CreatePolicyRequest {
        name: "Standard License".to_string(),
        authentication_strategy: Some(AuthenticationStrategy::License),
        duration: Some(365), // days
        max_machines: Some(3),
        ..Default::default()
    }).await?;
    
    Ok(())
}
```

### License Management

```rust
use keygen_rs::license::{License, CreateLicenseRequest};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a new license
    let license = License::create(CreateLicenseRequest {
        policy_id: "POLICY_ID".to_string(),
        user_email: Some("user@example.com".to_string()),
        ..Default::default()
    }).await?;
    
    // Suspend a license
    license.suspend().await?;
    
    // Reinstate a license
    license.reinstate().await?;
    
    Ok(())
}
```

### Service Introspection

```rust
use keygen_rs::service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Check if the service is available
    service::ping().await?;
    
    // Get detailed service information
    let info = service::get_service_info().await?;
    println!("API Version: {}", info.api_version);
    
    // Check if a specific feature is supported
    if service::supports_product_code().await? {
        println!("Product codes are supported!");
    }
    
    Ok(())
}
```

## Examples

For more detailed examples, please refer to the `examples` directory in the repository:

- **License Examples**: `/examples/license/` - License validation, activation, and management
- **Machine Examples**: `/examples/machine/` - Machine activation and management
- **Product Examples**: `/examples/product/` - Product CRUD operations (admin only)
- **Policy Examples**: `/examples/policy/` - Policy management (admin only)
- **User Examples**: `/examples/user/` - User management (admin only)
- **Token Examples**: `/examples/token/` - Token management (admin only)
- **Configuration Examples**: `/examples/config_examples.rs` - Different configuration approaches
- **Service Info**: `/examples/service_info.rs` - Service introspection

## Testing

When implementing a testing strategy for your licensing integration, we recommend mocking the Keygen API responses. This is especially important for CI/CD environments to prevent unnecessary load on Keygen's servers and to stay within your account's daily request limits.
You can use crates like `mockito` or `wiremock` to mock HTTP responses in your tests.

## Inspired by

- [keygen-go](https://github.com/keygen-sh/keygen-go)
- [tauri-plugin-keygen](https://github.com/bagindo/tauri-plugin-keygen)

## License

This project is licensed under the MIT License.
