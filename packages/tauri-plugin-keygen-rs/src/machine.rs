use keygen_rs::machine::Machine;
use tauri::{api::os::locale, webview_version};

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
    pub machine: Option<Machine>,
}

impl MachineState {
    pub(crate) fn new(app_name: String, app_version: String) -> Self {
        let fingerprint = machine_uid::get().unwrap_or("".into());
        let name = whoami::devicename();

        let os_name = format!("{}", whoami::platform());
        let os_version = whoami::distro().to_string();
        let arch = format!("{}", whoami::arch());
        let platform = format!("{} - {} - {}", os_name, os_version, arch);

        let engine_name = ENGINE_NAME.to_string();
        let engine_version = webview_version().unwrap_or_default();
        let locale = locale().unwrap_or_default();
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
            machine: None,
        }
    }
}
