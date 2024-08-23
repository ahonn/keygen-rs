use std::env;

use dotenv::dotenv;

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
