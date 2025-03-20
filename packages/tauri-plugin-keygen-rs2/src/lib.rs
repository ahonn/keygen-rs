use error::Error;
use keygen_rs::{config::KeygenConfig, license_file::LicenseFile, machine_file::MachineFile};
use lazy_static::lazy_static;
use license::LicenseState;
use machine::MachineState;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime, State,
};
use tokio::sync::Mutex;

mod commands;
pub mod error;
pub mod license;
pub mod machine;
mod utils;

pub type Result<T> = std::result::Result<T, Error>;

lazy_static! {
    static ref LISTENERS: Mutex<Vec<Box<dyn Fn(&LicenseState) + Send + Sync + 'static>>> =
        Mutex::new(Vec::new());
}

pub async fn add_license_listener<F>(listener: F)
where
    F: Fn(&LicenseState) + Send + Sync + 'static,
{
    let mut listeners = LISTENERS.lock().await;
    listeners.push(Box::new(listener));
}

pub(crate) async fn notify_license_listeners(state: &LicenseState) {
    let listeners = LISTENERS.lock().await;
    for listener in listeners.iter() {
        listener(state);
    }
}

pub trait AppHandleExt {
    fn get_license_state(&self) -> State<'_, Mutex<LicenseState>>;
    fn load_license_file(&self, key: &str) -> Result<Option<LicenseFile>>;
    fn remove_license_file(&self) -> Result<()>;
    fn get_machine_state(&self) -> State<'_, Mutex<MachineState>>;
    fn load_machine_file(&self, key: &str) -> Result<Option<MachineFile>>;
    fn remove_machine_file(&self) -> Result<()>;
}

impl<R: Runtime> AppHandleExt for tauri::AppHandle<R> {
    fn get_license_state(&self) -> State<'_, Mutex<LicenseState>> {
        self.state::<Mutex<LicenseState>>()
    }

    fn load_license_file(&self, key: &str) -> Result<Option<LicenseFile>> {
        LicenseState::load_license_file(self, key)
    }

    fn remove_license_file(&self) -> Result<()> {
        LicenseState::remove_license_file(self)
    }

    fn get_machine_state(&self) -> State<'_, Mutex<MachineState>> {
        self.state::<Mutex<MachineState>>()
    }

    fn load_machine_file(&self, key: &str) -> Result<Option<MachineFile>> {
        MachineState::load_machine_file(self, key)
    }

    fn remove_machine_file(&self) -> Result<()> {
        MachineState::remove_machine_file(self)
    }
}

pub struct Builder {
    account: String,
    product: String,
    public_key: String,
    api_url: Option<String>,
    api_version: Option<String>,
    api_prefix: Option<String>,
}

impl Builder {
    pub fn new(
        account: impl Into<String>,
        product: impl Into<String>,
        public_key: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            product: product.into(),
            public_key: public_key.into(),
            api_url: None,
            api_version: None,
            api_prefix: None,
        }
    }

    pub fn api_url(mut self, api_url: impl Into<String>) -> Self {
        self.api_url = Some(api_url.into());
        self
    }

    pub fn api_version(mut self, api_version: impl Into<String>) -> Self {
        self.api_version = Some(api_version.into());
        self
    }

    pub fn api_prefix(mut self, api_prefix: impl Into<String>) -> Self {
        self.api_prefix = Some(api_prefix.into());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        keygen_rs::config::set_config(KeygenConfig {
            api_url: self.api_url.unwrap_or("https://api.keygen.sh".to_string()),
            api_version: self.api_version.unwrap_or("1.7".to_string()),
            api_prefix: self.api_prefix.unwrap_or("v1".to_string()),
            account: self.account,
            product: self.product,
            public_key: Some(self.public_key),
            ..Default::default()
        });

        PluginBuilder::new("keygen-rs2")
            .invoke_handler(tauri::generate_handler![
                commands::get_license,
                commands::is_license_valid,
                commands::get_license_key,
                commands::validate_key,
                commands::activate,
                commands::deactivate,
                commands::checkout_license,
                commands::checkout_machine,
                commands::reset_license,
                commands::get_license_metadata,
            ])
            .setup(move |app_handle, _api| {
                let app_name = app_handle.package_info().name.clone();
                let app_version = app_handle.package_info().version.to_string();

                let machine_state = MachineState::new(app_name, app_version);
                app_handle.manage(Mutex::new(machine_state));

                let license_state = LicenseState::load(app_handle);
                match license_state {
                    Ok(license_state) => {
                        app_handle.manage(Mutex::new(license_state));
                    }
                    Err(err) => {
                        if let Error::KeygenError(e) = err {
                            match e {
                                keygen_rs::errors::Error::LicenseFileExpired(dataset) => {
                                    let license = dataset.license.clone();
                                    let license_state = LicenseState {
                                        key: Some(license.key.clone()),
                                        license: Some(license),
                                        valid: false,
                                    };
                                    app_handle.manage(Mutex::new(license_state));
                                }
                                keygen_rs::errors::Error::MachineFileExpired(dataset) => {
                                    let license = dataset.license.clone();
                                    let license_state = LicenseState {
                                        key: Some(license.key.clone()),
                                        license: Some(license),
                                        valid: false,
                                    };
                                    app_handle.manage(Mutex::new(license_state));
                                }
                                _ => {
                                    let license_state = LicenseState::default();
                                    app_handle.manage(Mutex::new(license_state));
                                }
                            }
                        }
                    }
                };

                Ok(())
            })
            .build()
    }
}
