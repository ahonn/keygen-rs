use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use keygen_rs::{machine::MachineCheckoutOpts, machine_file::MachineFile};
use tauri::{webview_version, AppHandle, Runtime};
#[cfg(mobile)]
use tauri_plugin_machine_uid::MachineUidExt;

use crate::{error::Error, utils::get_app_keygen_path, AppHandleExt, Result};

#[cfg(mobile)]
static ENGINE_NAME: &str = "WRY";

#[cfg(target_os = "linux")]
static ENGINE_NAME: &str = "WebKit";

#[cfg(target_os = "macos")]
static ENGINE_NAME: &str = "WebKit";

#[cfg(target_os = "windows")]
static ENGINE_NAME: &str = "WebView2";

#[derive(Debug, Clone)]
pub struct MachineState {
    pub name: String,
    pub fingerprint: String,
    pub platform: String,
    pub user_agent: String,
}

impl MachineState {
    #[cfg(desktop)]
    pub fn get_fingerprint() -> String {
        machine_uid::get().unwrap_or_default()
    }

    #[cfg(desktop)]
    pub fn get_fingerprint_app<R: Runtime>(_app_handle: &AppHandle<R>) -> String {
        Self::get_fingerprint()
    }

    #[cfg(mobile)]
    pub fn get_fingerprint() -> String {
        panic!(
            r#"On mobile, you should use the `get_fingerprint_app` method instead of `get_fingerprint`."#
        );
    }

    #[cfg(mobile)]
    pub fn get_fingerprint_app<R: Runtime>(app_handle: &AppHandle<R>) -> String {
        let Some(state) = app_handle.try_machine_uid() else {
            panic!(
                r#"tauri-plugin-machine-uid is not initialized but is required to get machine fingerprints on mobile.
You should add it to your application and initialize it before tauri-plugin-keygen-rs2."#
            );
        };

        state
            .get_machine_uid()
            .ok()
            .and_then(|uid| uid.id)
            .unwrap_or_default()
    }

    pub(crate) fn new<R: Runtime>(
        app_handle: &AppHandle<R>,
        app_name: String,
        app_version: String,
    ) -> Self {
        let fingerprint = Self::get_fingerprint_app(app_handle);
        let name = whoami::devicename();

        let os_name = format!("{}", whoami::platform());
        let os_version = whoami::distro().to_string();
        let arch = format!("{}", whoami::arch());
        let platform = format!("{} - {} - {}", os_name, os_version, arch);

        let engine_name = ENGINE_NAME.to_string();
        let engine_version = webview_version().unwrap_or_default();
        let locale = sys_locale::get_locale().unwrap_or_default();
        let user_agent = format!(
            "{}/{} {}/{} {}/{} {}",
            app_name, app_version, os_name, os_version, engine_name, engine_version, locale
        );

        keygen_rs::config::set_platform(&platform);
        keygen_rs::config::set_user_agent(&user_agent);

        Self {
            name,
            fingerprint,
            platform,
            user_agent,
        }
    }

    pub async fn checkout<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        options: &MachineCheckoutOpts,
    ) -> Result<MachineFile> {
        let license_state = app_handle.get_license_state();
        let license_state = license_state.lock().await;
        if let Some(license) = &license_state.license {
            log::info!("Checking out machine file: {}", self.fingerprint);
            let machine = license.machine(&self.fingerprint).await?;
            let machine_file = machine.checkout(options).await?;
            Self::save_machine_file(app_handle, &machine_file)?;
            Ok(machine_file)
        } else {
            Err(Error::NoLicenseError)
        }
    }

    pub fn load_machine_file<R: Runtime>(
        app_handle: &AppHandle<R>,
        key: &str,
    ) -> Result<Option<MachineFile>> {
        let path = Self::get_machine_file_path(app_handle)?;
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let machine_file = MachineFile::from_cert(key, &content)?;
        Ok(Some(machine_file))
    }

    pub fn remove_machine_file<R: Runtime>(app_handle: &AppHandle<R>) -> Result<()> {
        let path = Self::get_machine_file_path(app_handle)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn get_machine_file_path<R: Runtime>(app_handle: &AppHandle<R>) -> Result<PathBuf> {
        let data_dir = get_app_keygen_path(app_handle)?;
        let path = data_dir.join("machine.lic");
        Ok(path)
    }

    fn save_machine_file<R: Runtime>(
        app_handle: &AppHandle<R>,
        machine_file: &MachineFile,
    ) -> Result<()> {
        let path = Self::get_machine_file_path(app_handle)?;
        let mut file = File::create(path)?;
        file.write_all(machine_file.certificate.as_bytes())?;
        Ok(())
    }
}
