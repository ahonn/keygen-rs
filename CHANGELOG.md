# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
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
