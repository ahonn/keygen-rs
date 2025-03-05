# Tauri Plugin keygen-rs2

Tauri V2 plugin for Keygen.sh licensing, based on the keygen-rs SDK.

> For Tauri v1, please use [tauri-plugin-keygen-rs](../tauri-plugin-keygen-rs).

### Sponsored by

<a href="https://keygen.sh?via=keygen-rs" style="margin-right: 10px">
    <img src="https://keygen.sh/images/logo-pill.png" width="200" alt="Keygen">
</a>
<a href="https://badgeify.app?ref=keygen-rs">
    <img src="https://badgeify.app/logo-pill.png" width="200" alt="Badgeify">
</a>

## Features

- License validation and management
- Machine-specific license activation and deactivation
- Real-time license state updates
- Rust and TypeScript APIs for seamless integration
- Customizable Keygen.sh API endpoint

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-keygen-rs2 = "0.4"
```

## Usage

1. In your Tauri app's `main.rs`, import and use the plugin:

```rust
use tauri_plugin_keygen_rs2::Builder;

fn main() {
tauri::Builder::default()
    .plugin(
        Builder::new(account, product, public_key)
        .api_url(api_url)
        .build()
    )
    // ... other configurations
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
```

2. You can use the plugin's API in your frontend code:

```typescript
import {
  getLicense,
  validateKey,
  deactivate,
  checkoutLicense,
  checkoutMachine,
} from 'tauri-plugin-keygen-rs-api2';

const license = await validateKey('YOUR_LICENSE_KEY');
```

3. Access the plugin state in Rust code:

```rust
tauri::Builder::default()
    // ... other configurations
    .setup(|app| {
        let app_handle = app.handle();

        tauri::async_runtime::block_on(async move {
            let license_state = app_handle.get_license_state();
            let license_state = license_state.lock().await;
            println!("License: {:?}", license_state.license);

            let machine_state = app_handle.get_machine_state();
            let machine_state = machine_state.lock().await;
            println!("Machine: {:?}", machine_state);

            tauri_plugin_keygen_rs2::add_license_listener(|state| {
                println!("License state change: {:?}", state);
            }).await;
        });
        Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

For more detailed examples, refer to the [examples](./examples) directory.

## Error Handling

The plugin uses a `KeygenError` class for error handling. Catch and handle these errors in your application as needed.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
