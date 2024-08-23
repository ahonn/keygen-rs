use crate::{error::InvokeError, license::LicenseState, machine::MachineState};
use tokio::sync::Mutex;

use keygen_rs::{
    component::Component,
    license::{License, LicenseCheckoutOpts},
    license_file::LicenseFile,
};
use tauri::{command, AppHandle, Runtime, State};

type Result<T> = std::result::Result<T, InvokeError>;

#[command]
pub async fn get_license(license_state: State<'_, Mutex<LicenseState>>) -> Result<Option<License>> {
    let license_state = license_state.lock().await;
    Ok(license_state.license.clone())
}

#[command]
pub async fn validate_key<R: Runtime>(
    key: String,
    components: Option<Vec<String>>,
    entitlements: Option<Vec<String>>,
    app_handle: AppHandle<R>,
    machine_state: State<'_, Mutex<MachineState>>,
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<License> {
    let mut license_state = license_state.lock().await;
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
    machine_state: State<'_, Mutex<MachineState>>,
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<()> {
    let mut machine_state = machine_state.lock().await;
    let license_state = license_state.lock().await;
    let machine = license_state
        .activate(
            &app_handle,
            &machine_state.fingerprint,
            &components.unwrap_or(vec![]),
        )
        .await?;
    machine_state.machine = Some(machine.clone());
    Ok(())
}

#[command]
pub async fn deactivate(
    machine_state: State<'_, Mutex<MachineState>>,
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<()> {
    let machine_state = machine_state.lock().await;
    let license_state = license_state.lock().await;
    license_state.deactivate(&machine_state.fingerprint).await?;
    Ok(())
}

#[command]
pub async fn checkout_license<R: Runtime>(
    ttl: Option<i64>,
    include: Option<Vec<String>>,
    app_handle: AppHandle<R>,
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<LicenseFile> {
    let license_state = license_state.lock().await;
    let options = LicenseCheckoutOpts { ttl, include };
    let license_file = license_state.checkout(&app_handle, &options).await?;
    Ok(license_file)
}
