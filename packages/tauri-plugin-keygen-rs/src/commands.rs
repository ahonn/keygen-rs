use crate::{errors::InvokeError, license::LicenseState};
use std::sync::Mutex;

use tauri::{command, State};

type Result<T> = std::result::Result<T, InvokeError>;

#[command]
pub fn get_license_key(
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<Option<String>> {
    let license_state = license_state.lock().unwrap();
    if let Some(license) = &license_state.license {
        Ok(Some(license.key.clone()))
    } else {
        Ok(None)
    }
}
