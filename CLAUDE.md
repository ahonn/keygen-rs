# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
# Run tests
cargo test -- --show-output
# or
devbox run test

# Build the library
cargo build
cargo build --release

# Run example
cargo run

# Build documentation
cargo doc
# or
devbox run build-docs

# Development environment
devbox shell  # Sets up Rust toolchain and Node.js
```

## Architecture Overview

This is an unofficial Rust SDK for Keygen.sh licensing service. The codebase consists of:

1. **Core SDK** (`/src/`): Pure Rust implementation
   - `client.rs`: HTTP client for Keygen API communication using reqwest
   - `license.rs`: License validation and management
   - `machine.rs`: Machine activation/deactivation functionality
   - `verifier.rs`: Offline license verification using ed25519 signatures
   - `config.rs`: Global SDK configuration management
   - `errors.rs`: Custom error types and handling

2. **Tauri Plugins** (`/packages/`):
   - `tauri-plugin-keygen-rs2/`: Tauri v2 plugin implementation
   - `tauri-plugin-keygen-rs/`: Tauri v1 plugin (commented out in workspace)

## Testing Approach

- Tests are inline with source code using `#[cfg(test)]` modules
- Uses `mockito` for mocking HTTP responses
- Run a single test: `cargo test test_name -- --show-output`
- All API calls should be mocked in tests to avoid hitting Keygen servers

## Key Development Patterns

- **Async/Await**: All API operations use Tokio runtime
- **Builder Pattern**: Used for configuration setup
- **Error Handling**: Custom error types in `errors.rs` with thiserror
- **API Client**: Centralized HTTP client in `client.rs` handles all API communication
- **Offline Verification**: Ed25519 signature verification for offline license checks

## Environment Setup

- Uses `.env` file for configuration during development
- Devbox manages Rust toolchain and Node.js versions
- TLS backend: Default is rustls, can switch to native-tls via features

## Important API Entry Points

### Core License Operations
- `keygen_rs::validate()`: Validates a license key
- `keygen_rs::verify()`: Verifies signed license keys offline
- `License::activate()`: Activates a machine for a license
- `Machine::deactivate()`: Deactivates a machine

### Management APIs (MVP)
- **Product Management**: `Product::create()`, `Product::list()`, `Product::get()`, `Product::update()`, `Product::delete()`
- **Policy Management**: `Policy::create()`, `Policy::list()`, `Policy::get()`, `Policy::update()`, `Policy::delete()`
- **License Management**: `License::create()`, `License::list()`, `License::get()`, `License::update()`, `License::delete()`, `License::suspend()`, `License::reinstate()`
- **Machine Management**: `Machine::list()`, `Machine::get()`, `Machine::update()`, `Machine::reset()`, `Machine::change_owner()`

### Configuration for Management Client
```rust
use keygen_rs::config::{self, KeygenConfig};

// Configure with Admin Token for management operations
config::set_config(KeygenConfig {
    api_url: "https://api.keygen.sh".to_string(), // or your custom domain
    account: "your-account-id".to_string(),
    token: Some("your-admin-token".to_string()), // Admin Token for full access
    ..KeygenConfig::default()
});
```

### Type-Safe Enums
- `DistributionStrategy`: Open, Closed, Licensed
- `Platform`: Windows, MacOs, Linux, Darwin, Android, Ios, Web
- `AuthenticationStrategy`: Token, License, Mixed, None
- `ExpirationStrategy`: RestrictAccess, RevokeAccess, MaintainAccess, AllowAccess