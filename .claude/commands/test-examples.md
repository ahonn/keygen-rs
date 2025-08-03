---
description: Run all keygen-rs examples to test SDK functionality
argument-hint: "[category]"
allowed-tools: ["Bash", "Read"]
---

# Test Examples Command

Run all keygen-rs examples to test SDK functionality. You can optionally specify a category to test only specific examples.

Categories: all, service, entitlement, license, machine, policy, product, token, user

**Arguments:** $ARGUMENTS

## Task Instructions

1. First check if .env file exists and contains required environment variables
2. Based on the category argument (default: "all"), run the appropriate examples:

### Service Examples (always run for "all" or "service"):
- `cargo run --example health_check`
- `cargo run --example ping_service`

### Entitlement Examples (run for "all" or "entitlement"):
- `cargo run --example create_entitlement --features token`
- `cargo run --example list_entitlements --features token`
- Create an entitlement, then test get/update/delete with the created ID
- Use `echo "yes"` for delete confirmation

### License Examples (run for "all" or "license"):
- `cargo run --example validate_license`
- `cargo run --example verify_offline_key`
- `cargo run --example create_license --features token`
- `cargo run --example list_licenses --features token`
- Test other license operations using created license IDs
- `KEYGEN_ACCOUNT_ID=$KEYGEN_ACCOUNT KEYGEN_TOKEN=$KEYGEN_ADMIN_TOKEN cargo run --example list_licenses_pagination --features token`

### Machine Examples (run for "all" or "machine"):
- `cargo run --example activate_machine`
- `KEYGEN_LICENSE_ID=<license-id> cargo run --example create_machine --features token`
- Test other machine operations using created machine IDs

### Policy Examples (run for "all" or "policy"):
- `KEYGEN_PRODUCT_ID=$KEYGEN_PRODUCT cargo run --example create_policy --features token`
- `cargo run --example list_policies --features token`
- Test other policy operations using created policy IDs

### Product Examples (run for "all" or "product"):
- `cargo run --example create_product --features token`
- `cargo run --example list_products --features token`
- Test other product operations using created product IDs

### Token Examples (run for "all" or "token"):
- `cargo run --example list_tokens --features token`
- `KEYGEN_TOKEN_ID=<token-id> cargo run --example get_token --features token`

### User Examples (run for "all" or "user"):
- `cargo run --example create_user --features token`
- `cargo run --example list_users --features token`
- Test other user operations using created user IDs
- Use `echo "yes"` for delete confirmation

## Important Notes

- Extract IDs from command outputs using grep/awk when needed
- Handle confirmation prompts with `echo "yes" |`
- Skip dangerous operations on production tokens
- Provide clear status updates for each category
- Handle errors gracefully and continue with other tests