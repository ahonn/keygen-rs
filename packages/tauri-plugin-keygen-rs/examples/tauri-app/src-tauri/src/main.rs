use std::env;

use dotenv::dotenv;
use tauri_plugin_keygen_rs::AppHandleExt;

fn main() {
    dotenv().ok();

    let api_url = env::var("KEYGEN_API_URL").unwrap_or("https://api.keygen.sh/v1".to_string());
    let account = env::var("KEYGEN_ACCOUNT").unwrap();
    let product = env::var("KEYGEN_PRODUCT").unwrap();
    let public_key = env::var("KEYGEN_PUBLIC_KEY").unwrap();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_keygen_rs::Builder::new(account, product, public_key)
                .api_url(api_url)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![])
        .setup(|app| {
            let app_handle = app.handle();

            tauri::async_runtime::block_on(async move {
                let license_state = app_handle.get_license_state();
                let license_state = license_state.lock().await;
                println!("License: {:?}", license_state.license);

                let machine_state = app_handle.get_machine_state();
                let machine_state = machine_state.lock().await;
                println!("Machine: {:?}", machine_state);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
