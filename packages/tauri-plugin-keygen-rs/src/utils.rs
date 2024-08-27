use std::{fs, path::PathBuf};

use tauri::{AppHandle, Runtime};

use crate::{error::Error, Result};

pub fn get_app_keygen_path<R: Runtime>(app_handle: &AppHandle<R>) -> Result<PathBuf> {
    let app_data_dir = app_handle
        .path_resolver()
        .app_data_dir()
        .ok_or_else(|| Error::PathResolveError("App data directory not found".into()))?;
    let app_keygen_path = app_data_dir.join("keygen");
    if !app_keygen_path.exists() {
        fs::create_dir_all(&app_keygen_path)?;
    }
    Ok(app_keygen_path)
}
