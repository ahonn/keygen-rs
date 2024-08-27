use crate::{error::InvokeError, AppHandleExt};

use keygen_rs::{
    component::Component,
    license::{License, LicenseCheckoutOpts},
    license_file::LicenseFile,
    machine::MachineCheckoutOpts,
    machine_file::MachineFile,
};
use tauri::{command, AppHandle, Runtime};

type Result<T> = std::result::Result<T, InvokeError>;

#[command]
pub async fn get_license<R: Runtime>(app_handle: AppHandle<R>) -> Result<Option<License>> {
    let license_state = app_handle.get_license_state();
    let license_state = license_state.lock().await;
    Ok(license_state.license.clone())
}

#[command]
pub async fn is_license_valid<R: Runtime>(app_handle: AppHandle<R>) -> Result<bool> {
    let license_state = app_handle.get_license_state();
    let license_state = license_state.lock().await;
    Ok(license_state.valid)
}

#[command]
pub async fn get_license_key<R: Runtime>(app_handle: AppHandle<R>) -> Result<String> {
    let license_state = app_handle.get_license_state();
    let license_state = license_state.lock().await;
    if let Some(key) = &license_state.key {
        Ok(key.to_string())
    } else {
        Ok("".to_string())
    }
}

#[command]
pub async fn validate_key<R: Runtime>(
    key: String,
    components: Option<Vec<String>>,
    entitlements: Option<Vec<String>>,
    app_handle: AppHandle<R>,
) -> Result<License> {
    let license_state = app_handle.get_license_state();
    let mut license_state = license_state.lock().await;

    let machine_state = app_handle.get_machine_state();
    let machine_state = machine_state.lock().await;

    let mut fingerprints = components.unwrap_or_else(|| vec![]);
    fingerprints.insert(0, machine_state.fingerprint.clone());

    let license = license_state
        .validate_key(
            &app_handle,
            &key,
            &fingerprints,
            &entitlements.unwrap_or(vec![]),
        )
        .await?;
    Ok(license)
}

#[command]
pub async fn activate<R: Runtime>(
    components: Option<Vec<Component>>,
    app_handle: AppHandle<R>,
) -> Result<()> {
    let license_state = app_handle.get_license_state();
    let mut license_state = license_state.lock().await;

    let machine_state = app_handle.get_machine_state();
    let machine_state = machine_state.lock().await;

    license_state
        .activate(
            &app_handle,
            &machine_state.fingerprint,
            &components.unwrap_or(vec![]),
        )
        .await?;
    Ok(())
}

#[command]
pub async fn deactivate<R: Runtime>(app_handle: AppHandle<R>) -> Result<()> {
    let license_state = app_handle.get_license_state();
    let mut license_state = license_state.lock().await;

    let machine_state = app_handle.get_machine_state();
    let machine_state = machine_state.lock().await;

    license_state
        .deactivate(&app_handle, &machine_state.fingerprint)
        .await?;
    Ok(())
}

#[command]
pub async fn checkout_license<R: Runtime>(
    ttl: Option<i64>,
    include: Option<Vec<String>>,
    app_handle: AppHandle<R>,
) -> Result<LicenseFile> {
    let license_state = app_handle.get_license_state();
    let license_state = license_state.lock().await;

    let options = LicenseCheckoutOpts { ttl, include };
    let license_file = license_state.checkout(&app_handle, &options).await?;
    Ok(license_file)
}

#[command]
pub async fn checkout_machine<R: Runtime>(
    ttl: Option<i64>,
    include: Option<Vec<String>>,
    app_handle: AppHandle<R>,
) -> Result<MachineFile> {
    let machine_state = app_handle.get_machine_state();
    let machine_state = machine_state.lock().await;

    let options = MachineCheckoutOpts { ttl, include };
    let machine_file = machine_state.checkout(&app_handle, &options).await?;
    Ok(machine_file)
}
