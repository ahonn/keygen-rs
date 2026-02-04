# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
## [0.9.1] - 2026-02-04

### Fixed

- Update signature headers and fix dead_code warnings ([55578ce](https://github.com/ahonn/keygen-rs/commit/55578ce7cdc5bc0000097532877de7225946b96d))

### Other

- Install Linux dependencies for tauri-plugin-keygen-rs2 ([c637ac2](https://github.com/ahonn/keygen-rs/commit/c637ac25c48318ea34e379455b5c9a91d8617c74))
- Update README for v0.9 release ([78a6908](https://github.com/ahonn/keygen-rs/commit/78a6908f28793f6b360fc55495191c130a69aad6))


## [0.9.0] - 2026-02-03

### Added

- Add Distribution API modules (Package, Artifact, Platform, Arch, Channel) ([a260869](https://github.com/ahonn/keygen-rs/commit/a260869c562bc4c247c5f145f91454ac9549d8ea))
- Add Release API support for software distribution ([468d4a4](https://github.com/ahonn/keygen-rs/commit/468d4a49302a2ad0ddb5b4da52d2a7d5a7236962))

### Fixed

- Remove redundant error wrapping in signature verification ([243bb14](https://github.com/ahonn/keygen-rs/commit/243bb14aa91944aca282f1e4ad265df4fa715fb1))
- Remove .pre-commit-config.yaml from .gitignore ([422de11](https://github.com/ahonn/keygen-rs/commit/422de1182f9cc6e25cc015dae3eff24a465add1e))
- Correct query parameter serialization for all List APIs ([1c5c0cc](https://github.com/ahonn/keygen-rs/commit/1c5c0cccb321a46b133a3712e85b6fcec7862301))
- Update create_artifact example to list/get operations ([58192cd](https://github.com/ahonn/keygen-rs/commit/58192cd81498014b2b46af2cc9507caa9ce68d2f))

### Other

- [**breaking**] Apply Rust best practices across codebase ([6786528](https://github.com/ahonn/keygen-rs/commit/67865289be404cbc66f8ae91bcbb7c9034caac0e))
- Rename create_artifact to get_artifact ([19e7372](https://github.com/ahonn/keygen-rs/commit/19e7372c8ab5b1da0bdb6101528e6164624ec6fe))
- Migrate from devbox to devenv ([af1db5e](https://github.com/ahonn/keygen-rs/commit/af1db5e059daeb7b232b0e99974bb0c3f2f0570f))


## [0.8.1] - 2025-08-26

### Fixed

- Update tauri plugin for async config state management ([b48fa11](https://github.com/ahonn/keygen-rs/commit/b48fa1124bc48e8c98ce0c7820234b80491ac862))

## [0.8.0] - 2025-08-26

### Added

- Add with_config pattern for validate and verify ([bc996c6](https://github.com/ahonn/keygen-rs/commit/bc996c6ad2e5e2a85e02eda3989fb2024b1324cc))
- Refactor to use with_config pattern ([2fa2bae](https://github.com/ahonn/keygen-rs/commit/2fa2bae04ddf055d1433d7e9de5b6f2258fc1c68))

## [0.7.4] - 2025-08-12

### Fixed

- Correct webhook signature algorithm values to match Keygen API ([b9a0506](https://github.com/ahonn/keygen-rs/commit/b9a050664560fb8b3c659fa4e683bc80320d6660))

## [0.7.3] - 2025-08-09

### Added

- Implement license usage tracking operations ([efaa308](https://github.com/ahonn/keygen-rs/commit/efaa308d21d65a3d1e49719c50b11a16229698fa))
- Implement complete Environment API with examples ([1369726](https://github.com/ahonn/keygen-rs/commit/1369726fa873dc06bb0ae72fd2758086a12cbeb3))
- Implement complete Group API with and examples ([240edf3](https://github.com/ahonn/keygen-rs/commit/240edf381e70b855d4406676034ff9c3c131abc5))
- Implement complete Component API with examples ([c76cb2a](https://github.com/ahonn/keygen-rs/commit/c76cb2ad2243186ce54f557b5a04ec7e9f41076d))
- Add webhook API implementation ([1817ede](https://github.com/ahonn/keygen-rs/commit/1817edebf9da9bc530500c2e965689bed23b691a))

### Fixed

- Replace fingerprint.clone() with std::slice::from_ref in examples ([f59964e](https://github.com/ahonn/keygen-rs/commit/f59964e029a3f2e79db4f3ac4acbe7eb78678097))

### Other

- Update GitHub release workflow ([c651155](https://github.com/ahonn/keygen-rs/commit/c6511555b7e56370f077dfb4f98c2e59d002a521))

## [0.7.2] - 2025-08-08

### Fixed

- Add TLS backend configuration support to tauri-plugin-keygen-rs2 ([d1ac890](https://github.com/ahonn/keygen-rs/commit/d1ac8905e37e5b7b8c7e47d974f8836da440b5ab))
- Configure tauri-plugin-keygen-rs2 to use workspace dependencies ([624297f](https://github.com/ahonn/keygen-rs/commit/624297f8847bb3274fc1927b6d702f2885115fec))

### Other

- Add CLAUDE.md with development guidance ([0e90396](https://github.com/ahonn/keygen-rs/commit/0e903960a8284b3270747cca7aa66681a2962156))

## [0.7.1] - 2025-08-03

### Fixed

- Resolve duplicate example name in Cargo.toml ([9840251](https://github.com/ahonn/keygen-rs/commit/984025196ba5575e6acd1b143c1e0fec53964b5e))
- Correct release-plz configuration format ([d72cd53](https://github.com/ahonn/keygen-rs/commit/d72cd534906f770a459e7ff05d976cd842c1af32))
- Resolve all clippy warnings and improve code quality ([f426b82](https://github.com/ahonn/keygen-rs/commit/f426b820d30d218049075d2d40089047c1370b46))
- Remove invalid path field from release-plz config ([280948b](https://github.com/ahonn/keygen-rs/commit/280948b8013a2ae3aea985d2c72da5313002ba89))

### Other

- Simplify test workflow to use only all-features configuration ([86ab59b](https://github.com/ahonn/keygen-rs/commit/86ab59b26eb850af05690c57a94d044cf0149328))
- Align CI and pre-commit clippy configuration ([ecf0994](https://github.com/ahonn/keygen-rs/commit/ecf0994b1ec5095dc3f3051b85cf6d0858ae7f6f))
- Setup prefligit pre-commit hooks ([6d0796a](https://github.com/ahonn/keygen-rs/commit/6d0796aceaff7acdd080f0dcc313a5d3fe29d5e4))
- Gitignore exclude .schema/ directory ([18a0fd2](https://github.com/ahonn/keygen-rs/commit/18a0fd24dbdf02086fedf443c053d0d4cbd6f245))
- Update test workflow configuration ([127eb0f](https://github.com/ahonn/keygen-rs/commit/127eb0f1cacaf7321b354414d1b6cc297545f9ea))
- Formatting all rust code ([d9c6afe](https://github.com/ahonn/keygen-rs/commit/d9c6afee87b69762e2dec594fd1347f62ce3f554))
- Add automated release workflow with release-plz ([97e0afe](https://github.com/ahonn/keygen-rs/commit/97e0afea5aee2d189dbc2bfbcfaf6b804ae28d5c))
