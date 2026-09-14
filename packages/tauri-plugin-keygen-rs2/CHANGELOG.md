# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.12.1] - 2026-09-14

### Other

- Updated the following local packages: keygen-rs
- 0.12.0 was not published to crates.io; this release includes its changes

## [0.12.0] - 2026-09-14

### Added

- Complete API and offline file support ([57c3ad2](https://github.com/ahonn/keygen-rs/commit/57c3ad2630267fffd0b2f4fb4a23efd577eebe02))
- [**breaking**] Complete Keygen 1.7+ API alignment ([bacc6f6](https://github.com/ahonn/keygen-rs/commit/bacc6f66f3fef874fdedd7e9589f94e6c0742a6d))
- [**breaking**] Introduce instance-scoped API configuration ([e50bf1d](https://github.com/ahonn/keygen-rs/commit/e50bf1d91e1332beb7a127f6613e1e9e711aa9e9))

### Other

- Remove the sponsor section from the READMEs ([77eef91](https://github.com/ahonn/keygen-rs/commit/77eef9110a145a4dac431eb975d0719186732bf7))


## [0.9.0] - 2026-02-03

### Other

- Tauri-plugin-keygen2-api v0.2.4 ([fbc3c5d](https://github.com/ahonn/keygen-rs/commit/fbc3c5d794fdfccac95b88543566b1a5a31cb8dd))
- Bump the tauri-plugin-deps group ([5294452](https://github.com/ahonn/keygen-rs/commit/52944524d5c98826d63ee4ff6eb962de9760633b))


## [0.8.1] - 2025-08-26

### Fixed

- Update tauri plugin for async config state management ([b48fa11](https://github.com/ahonn/keygen-rs/commit/b48fa1124bc48e8c98ce0c7820234b80491ac862))

## [0.8.0] - 2025-08-26

### Added

- Add with_config pattern for validate and verify ([bc996c6](https://github.com/ahonn/keygen-rs/commit/bc996c6ad2e5e2a85e02eda3989fb2024b1324cc))
- Refactor to use with_config pattern ([2fa2bae](https://github.com/ahonn/keygen-rs/commit/2fa2bae04ddf055d1433d7e9de5b6f2258fc1c68))

## [0.7.5] - 2025-08-18

### Added

- Re-export keygen-rs crate in tauri-plugin-keygen-rs2 ([709fd54](https://github.com/ahonn/keygen-rs/commit/709fd54620982382442156f4ebcd307e41d2efc9))

## [0.7.2] - 2025-08-08

### Fixed

- Add TLS backend configuration support to tauri-plugin-keygen-rs2 ([d1ac890](https://github.com/ahonn/keygen-rs/commit/d1ac8905e37e5b7b8c7e47d974f8836da440b5ab))

## [0.7.1] - 2025-08-03

### Fixed

- Resolve all clippy warnings and improve code quality ([f426b82](https://github.com/ahonn/keygen-rs/commit/f426b820d30d218049075d2d40089047c1370b46))

