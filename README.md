# Unofficial Keygen Rust SDK

[![Crates.io](https://img.shields.io/crates/v/keygen-rs.svg)](https://crates.io/crates/keygen-rs)
[![Documentation](https://docs.rs/keygen-rs/badge.svg)](https://docs.rs/keygen-rs)

The `keygen-rs` crate is an unofficial Rust SDK for integrating with the [keygen.sh](https://keygen.sh) licensing service.

## Features

- **License Management**: Validate, activate, and verify licenses offline
- **Machine Management**: Activate, deactivate, and manage machines with heartbeat monitoring
- **Administrative APIs**: Full CRUD operations for products, policies, licenses, users, and tokens (requires admin token)
- **Distribution APIs**: Manage releases, packages, artifacts, platforms, architectures, and channels
- **Offline Verification**: Verify signed license keys without internet connectivity
- **Type Safety**: Strongly-typed enums for all API options (LicenseStatus, HeartbeatStatus, etc.)
- **Service Introspection**: Check API availability and the observed API contract
- **Security**: Sensitive data is automatically zeroed from memory using `zeroize`

## Installing

Add this to your `Cargo.toml`:

```toml
[dependencies]
keygen-rs = "0.12"
```

### Feature Flags

The SDK uses feature flags to minimize binary size:

- **`license-key`** (default): End-user features for license validation and machine activation
- **`token`**: Administrative features requiring token authentication

```toml
# For end-user features only (default)
keygen-rs = "0.12"

# For administrative features
keygen-rs = { version = "0.12", features = ["token"] }

# For both end-user and administrative features
keygen-rs = { version = "0.12", features = ["license-key", "token"] }
```

## Tauri Plugin

Tauri plugins for this SDK are available:

- [tauri-plugin-keygen-rs](./packages/tauri-plugin-keygen-rs) for Tauri v1
- [tauri-plugin-keygen-rs2](./packages/tauri-plugin-keygen-rs2) for Tauri v2

These plugins provide an easy way to integrate Keygen licensing into your Tauri applications. For more information, check the plugins' respective READMEs.

## Config

### KeygenConfig

Use an instance client when possible. Each client pins its own API contract, so API 1.7 and 1.8 clients can safely coexist in one process. The global configuration API remains available as a compatibility facade.

#### For End Users (License Key Authentication)

```rust
use keygen_rs::{ApiContractVersion, KeygenClient};

let client = KeygenClient::builder()
    .account("YOUR_KEYGEN_ACCOUNT_ID")
    .product("YOUR_KEYGEN_PRODUCT_ID")
    .license_key("A_KEYGEN_LICENSE_KEY")
    .public_key("YOUR_KEYGEN_PUBLIC_KEY")
    .api_contract_version(ApiContractVersion::V1_8)
    .build()?;
```

#### For Administrators (Token Authentication)

```rust
use keygen_rs::KeygenClient;

let client = KeygenClient::builder()
    .account("YOUR_KEYGEN_ACCOUNT_ID")
    .token("YOUR_ADMIN_TOKEN")
    .build()?;
```

#### Custom Configuration

```rust
use keygen_rs::{config::{self, KeygenConfig}, ApiContractVersion};

config::set_config(KeygenConfig {
    api_url: "https://api.keygen.sh".to_string(), // or your custom domain
    api_version: ApiContractVersion::V1_8,
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
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::LicenseValidationRequest,
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
    let request = LicenseValidationRequest::for_fingerprint(fingerprint);
    let validation = keygen_rs::validate(&request).await?;
    println!("License validation: {} ({})", validation.meta.valid, validation.meta.code);

    Ok(())
}
```

### Activate a Machine

To activate a machine for a license:

```rust
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::LicenseValidationRequest,
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
    let request = LicenseValidationRequest::for_fingerprint(fingerprint.clone());
    let validation = keygen_rs::validate(&request).await?;
    if validation.meta.valid {
        println!("License validated successfully");
    } else if matches!(
        validation.meta.code.as_str(),
        "NO_MACHINE" | "NO_MACHINES" | "FINGERPRINT_SCOPE_MISMATCH"
    ) && validation.license.is_some() {
        let license = validation.license.unwrap();
        let machine = license.activate(&fingerprint, &[]).await?;
        println!("License activated successfully: {:?}", machine);
    } else {
        println!("License validation failed: {}", validation.meta.detail);
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

Transport and API failures return `Error`; invalid licenses are returned as validation metadata:

```rust
use keygen_rs::{
    config::{self, KeygenConfig},
    errors::Error,
    license::LicenseValidationRequest,
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
    let request = LicenseValidationRequest::for_fingerprint(fingerprint.clone());
    let validation = keygen_rs::validate(&request).await?;
    match (validation.meta.valid, validation.license) {
        (true, Some(license)) => println!("License is valid: {:?}", license),
        (false, Some(license)) => {
            println!("License is invalid: {}", validation.meta.code);
            let machine = license.activate(&fingerprint, &[]).await?;
            println!("Machine activated: {:?}", machine);
        }
        (_, None) => println!("License not found"),
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
    println!("API Version: {:?}", info.api_version);
    
    Ok(())
}
```

### Distribution APIs

Manage software releases and artifacts:

```rust
use keygen_rs::release::{Release, CreateReleaseRequest, ListReleasesParams};
use keygen_rs::artifact::Artifact;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Create a new release
    let release = Release::create(CreateReleaseRequest {
        name: Some("v1.0.0".to_string()),
        version: "1.0.0".to_string(),
        channel: Some("stable".to_string()),
        ..Default::default()
    }).await?;

    // List releases
    let releases = Release::list(Some(ListReleasesParams {
        product: Some("PRODUCT_ID".to_string()),
        ..Default::default()
    })).await?;

    // Get artifacts for a release
    let artifacts = Artifact::list(Some(ListArtifactsParams {
        release: Some(release.id.clone()),
        ..Default::default()
    })).await?;

    Ok(())
}
```

## Examples

For more detailed examples, please refer to the `examples` directory in the repository:

- **License Examples**: `/examples/license/` - License validation, activation, and management
- **Machine Examples**: `/examples/machine/` - Machine activation, heartbeat monitoring, and management
- **Product Examples**: `/examples/product/` - Product CRUD operations (admin only)
- **Policy Examples**: `/examples/policy/` - Policy management (admin only)
- **User Examples**: `/examples/user/` - User management (admin only)
- **Token Examples**: `/examples/token/` - Token management (admin only)
- **Release Examples**: `/examples/release/` - Release lifecycle management (admin only)
- **Artifact Examples**: `/examples/artifact/` - Artifact management (admin only)
- **Package Examples**: `/examples/package/` - Package management (admin only)
- **Environment Examples**: `/examples/environment/` - Environment management (admin only)
- **Webhook Examples**: `/examples/webhook_endpoint/` and `/examples/webhook_event/` - Webhook management (admin only)
- **Service Examples**: `/examples/service/` - Service introspection

## Testing

When implementing a testing strategy for your licensing integration, we recommend mocking the Keygen API responses. This is especially important for CI/CD environments to prevent unnecessary load on Keygen's servers and to stay within your account's daily request limits.
You can use crates like `mockito` or `wiremock` to mock HTTP responses in your tests.

## Inspired by

- [keygen-go](https://github.com/keygen-sh/keygen-go)
- [tauri-plugin-keygen](https://github.com/bagindo/tauri-plugin-keygen)

## License

This project is licensed under the MIT License.
