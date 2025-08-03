# Release-plz Usage Guide

Release-plz is an automated release tool for Rust projects that automatically updates version numbers, generates CHANGELOGs, creates GitHub Releases, and publishes to crates.io.

## Workflow

### 1. Automated Process (Recommended)

When you push code to the `master` branch, release-plz will automatically:

1. **Detect Changes**: Analyze commits since the last release
2. **Create PR**: If there are changes to release, automatically create a PR that:
   - Updates version numbers in `Cargo.toml`
   - Generates/updates `CHANGELOG.md`
   - Commit message format: `bump: v0.7.1`
3. **Merge & Release**: When you merge this PR, it will automatically:
   - Create Git tags
   - Create GitHub Release
   - Publish to crates.io
   - Publish Tauri plugin to NPM

### 2. Versioning Rules

Release-plz automatically determines version bumps based on commit types:

- `feat:` → Minor version +1 (0.7.0 → 0.8.0)
- `fix:` → Patch version +1 (0.7.0 → 0.7.1)
- `feat!:` or `BREAKING CHANGE:` → Major version +1 (0.7.0 → 1.0.0)

### 3. Commit Convention

Use Conventional Commits:

```bash
# New feature
git commit -m "feat: add support for offline verification"

# Bug fix
git commit -m "fix: correct license validation logic"

# Breaking change
git commit -m "feat!: change API response format"

# With scope
git commit -m "feat(machine): add heartbeat functionality"

# Multi-line description
git commit -m "fix: resolve memory leak in verification

The verifier was not properly releasing memory after
signature validation, causing gradual memory increase."
```

### 4. Manual Trigger

To manually trigger a release:

1. Go to GitHub Actions page
2. Select "Release" workflow
3. Click "Run workflow"
4. Select `master` branch
5. Click "Run workflow" button

### 5. Local Preview

Preview changes locally:

```bash
# Install release-plz
cargo install release-plz

# Preview release
release-plz release --dry-run

# Preview version updates
release-plz update --dry-run
```

### 6. Skip Release

To skip version updates for certain commits:

```bash
# Add [skip ci] to commit message
git commit -m "docs: update README [skip ci]"

# Or use chore type (doesn't trigger version update by default)
git commit -m "chore: update dev dependencies"
```

### 7. Multi-Package Workspace

This project contains multiple packages. Release-plz will:

- Manage each package version independently
- Only update packages with changes
- Keep dependency versions in sync

### 8. Custom Release Notes

You can customize release notes in the PR description:

```markdown
## Release Notes

### Highlights
- 🚀 Added offline verification support
- 🐛 Fixed memory leak issue
- 📚 Improved documentation

### Breaking Changes
- API response format changed
```

### 9. Rollback Release

To rollback a release:

```bash
# 1. Revert code
git revert <commit-hash>

# 2. Create fix version
git commit -m "fix: revert breaking changes"

# 3. Push to trigger new version
git push origin master
```

## FAQ

### Q: Why didn't my commit trigger a version update?

A: Check the following:
- Is the commit using conventional commit format?
- Are you using non-versioning types like `chore:`, `docs:`, `style:`?
- Did you add `[skip ci]`?

### Q: How to modify auto-generated CHANGELOG?

A: You can directly edit CHANGELOG.md in the release-plz PR and commit to the same PR.

### Q: How to publish pre-release versions?

A: Manually set pre-release version in `Cargo.toml`:
```toml
version = "0.8.0-beta.1"
```

### Q: What to do if release fails?

A: 
1. Check GitHub Actions logs
2. Verify `CARGO_REGISTRY_TOKEN` and `NPM_TOKEN` are correctly set
3. Fix issues manually and re-run workflow

## Configuration Reference

See `release-plz.toml` for complete configuration options. Key configurations:

- `changelog_include`: Commit types to include in CHANGELOG
- `git_tag_name`: Git tag format
- `release_commits_message`: Release commit message format

## Best Practices

1. **Keep Commits Clear**: Each commit should do one thing
2. **Use Semantic Messages**: Accurately describe changes
3. **Merge PRs Promptly**: Avoid keeping version update PRs open too long
4. **Release Regularly**: Recommend releasing every 1-2 weeks
5. **Check CI Status**: Ensure all tests pass before merging release PR