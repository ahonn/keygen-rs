# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**keygen-rs** is an unofficial Rust SDK for integrating with the [keygen.sh](https://keygen.sh) licensing service. It provides both end-user licensing functionality and administrative APIs for license management.

## Development Commands

### Build & Test
```bash
# Standard build and test
cargo build
cargo test --all-features --verbose

# Feature-specific testing
cargo test                           # license-key features only
cargo test --features token          # admin features
cargo test --all-features           # all features

# Linting & formatting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Documentation
cargo doc --no-deps --all-features
```

### Devbox Environment
```bash
# Using devbox scripts
devbox run test      # cargo test -- --show-output
devbox run build-docs # cargo doc
```

## Architecture

### Feature Flag System
The codebase uses Cargo feature flags for modularity:

- **`license-key`** (default): End-user features (validation, activation)
- **`token`**: Administrative features (CRUD operations, user management)
- **`rustls`** (default): Uses rustls for TLS
- **`native-tls`**: Alternative TLS backend

### Core Module Structure

**Public API Modules:**
- `config.rs` - Global configuration with thread-safe lazy static setup
- `license.rs` - Core license validation, verification, and management
- `machine.rs` - Machine activation, deactivation, and management
- `service.rs` - Service introspection and health checks
- `entitlement.rs` - License entitlement management
- `errors.rs` - Comprehensive error handling with thiserror

**Admin Modules (token feature):**
- `product.rs`, `policy.rs`, `user.rs`, `token.rs` - Administrative CRUD operations

**Internal Modules:**
- `client.rs` - HTTP client wrapper with authentication
- `verifier.rs` - Ed25519 cryptographic signature verification
- `decryptor.rs` - License file decryption
- `certificate.rs` - Certificate handling

### Key Patterns

- **Thread-safe global configuration** using `lazy_static`
- **Strongly-typed enums** for all API options and states
- **Feature-gated compilation** to prevent misuse of admin functions
- **Comprehensive error types** with detailed context for licensing scenarios
- **Offline capabilities** with license file caching and fingerprint-based activation

## Dependencies

- **HTTP**: `reqwest` with rustls-tls, 30-second timeout default
- **Async**: `tokio` with full features
- **Serialization**: `serde` + `serde_json` for JSON API communication
- **Cryptography**: `ed25519-dalek`, `aes-gcm`, `sha2`
- **Time**: `chrono` for date/time handling

## Testing Strategy

- Use `cargo test --all-features` for comprehensive testing
- Examples in `examples/` directory serve as integration tests
- Recommend mocking Keygen API responses for CI/CD testing
- Unit tests focus on offline functionality and error handling

## Release Process

- Uses **release-plz** for automated semantic versioning
- Conventional commits drive version bumps:
  - `feat:` → Minor version
  - `fix:` → Patch version
  - `feat!:` → Major version
- Auto-generates changelogs and publishes to crates.io

## Workspace Structure

- **Main crate**: Core keygen-rs library
- **Tauri plugins**: `packages/tauri-plugin-keygen-rs*` for desktop app integration