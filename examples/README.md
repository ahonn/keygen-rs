# Examples

This directory contains examples for using the keygen-rs library with different feature configurations.

## Feature Flags

The library supports two main feature flags:

- **`client`** (default) - End-user features like license validation, machine activation
- **`admin`** - Administrator features like product/policy/license/user/machine management

## Running Examples

### Client Examples (default features)
```bash
# Run validation example
cargo run --example validate_license

# Run activation example  
cargo run --example activate_machine
```

### Admin Examples (requires admin feature)
```bash
# Run product management example
cargo run --features=admin --example product/create_product

# Run license management example
cargo run --features=admin --example license/create_license

# Run user management example
cargo run --features=admin --example user/create_user
```

## Directory Structure

### `/license/` - License Management (admin feature required)
- `create_license.rs` - Create new licenses
- `list_licenses.rs` - List all licenses
- `suspend_license.rs` - Suspend licenses
- `reinstate_license.rs` - Reinstate suspended licenses
- `checkout_license.rs` - License checkout (client feature)
- `validate_license.rs` - License validation (client feature)
- `read_license_metadata.rs` - Read license metadata (client feature)
- `verify_offline_key.rs` - Verify offline keys (client feature)

### `/machine/` - Machine Management
- `list_machines.rs` - List machines (admin feature)
- `reset_machine.rs` - Reset machine heartbeat (admin feature)
- `activate_machine.rs` - Activate machines (client feature)
- `deactivate_machine.rs` - Deactivate machines (client feature)
- `checkout_machine.rs` - Machine checkout (client feature)
- `start_heartbeat.rs` - Start heartbeat (client feature)

### `/product/` - Product Management (admin feature required)
- `create_product.rs` - Create products
- `list_products.rs` - List products

### `/policy/` - Policy Management (admin feature required)
- `create_policy.rs` - Create policies
- `list_policies.rs` - List policies

### `/user/` - User Management (admin feature required)
- `create_user.rs` - Create users
- `list_users.rs` - List users

## Environment Variables

All examples require these environment variables:

```bash
export KEYGEN_ACCOUNT="your-account-id"
export KEYGEN_API_URL="https://api.keygen.sh"  # optional, defaults to this
```

### For Client Examples:
```bash
export KEYGEN_LICENSE_KEY="your-license-key"
export KEYGEN_PUBLIC_KEY="your-public-key"
export KEYGEN_PRODUCT="your-product-id"
```

### For Admin Examples:
```bash
export KEYGEN_ADMIN_TOKEN="your-admin-token"
```

### For Specific Examples:
```bash
# For creating licenses
export POLICY_ID="your-policy-id"

# For machine/license operations
export MACHINE_ID="your-machine-id"  
export LICENSE_ID="your-license-id"
```

## Client vs Admin Features

### Client Features (End Users)
- License validation and verification
- Machine activation/deactivation
- License/machine checkout
- Heartbeat functionality
- Offline key verification

### Admin Features (Administrators) 
- Product CRUD operations
- Policy CRUD operations
- License management (create, list, suspend, etc.)
- User management (create, list, ban, etc.)
- Machine management (list, reset, transfer ownership)

## Tips

1. **Feature Selection**: Use only the features you need to reduce binary size
2. **Security**: Never include admin tokens in client applications
3. **Testing**: Use different tokens to test different permission levels
4. **Environment**: Consider using `.env` files for local development