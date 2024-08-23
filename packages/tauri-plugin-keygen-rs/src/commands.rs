use crate::{error::InvokeError, license::LicenseState, machine::MachineState};
use tokio::sync::Mutex;

use keygen_rs::{component::Component, license::License};
use tauri::{command, State};

type Result<T> = std::result::Result<T, InvokeError>;

#[command]
pub async fn get_license(license_state: State<'_, Mutex<LicenseState>>) -> Result<Option<License>> {
    let license_state = license_state.lock().await;
    Ok(license_state.license.clone())
}

#[command]
pub async fn validate_key(
    key: String,
    fingerprints: Vec<String>,
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<License> {
    let mut license_state = license_state.lock().await;
    let license = license_state.validate_key(&key, &fingerprints).await?;
    Ok(license)
}

#[command]
pub async fn activate(
    components: Vec<Component>,
    machine_state: State<'_, Mutex<MachineState>>,
    license_state: State<'_, Mutex<LicenseState>>,
) -> Result<()> {
    let mut machine_state = machine_state.lock().await;
    let license_state = license_state.lock().await;
    let machine = license_state
        .activate_machine(&machine_state.fingerprint, &components)
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
    license_state.deactivate_machine(&machine_state.fingerprint).await?;
    Ok(())
}
