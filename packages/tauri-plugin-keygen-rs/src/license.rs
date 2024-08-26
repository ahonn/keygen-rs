use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::{
    error::Error, machine::MachineState, utils::get_app_keygen_path, AppHandleExt, Result,
};
use keygen_rs::{
    component::Component,
    errors::Error as KeygenError,
    license::{License, LicenseCheckoutOpts},
    license_file::LicenseFile,
    machine::Machine,
};
use tauri::{AppHandle, Runtime};

#[derive(Debug, Clone, Default)]
pub struct LicenseState {
    pub key: Option<String>,
    pub license: Option<License>,
}

impl LicenseState {
    pub async fn validate_key<R: Runtime>(
        &mut self,
        app_handle: &AppHandle<R>,
        key: &str,
        fingerprints: &[String],
        entitlements: &[String],
    ) -> Result<License> {
        keygen_rs::config::set_license_key(key);
        let license = keygen_rs::validate(fingerprints, entitlements).await;
        if let Ok(license) = license {
            self.license = Some(license.clone());
            Self::save_license_key_cache(app_handle, &license)?;
            Ok(license)
        } else {
            let error = license.unwrap_err();
            match error {
                KeygenError::LicenseNotActivated { ref license, .. } => {
                    self.license = Some(license.clone());
                    return Err(error.into());
                }
                _ => {}
            }
            Err(error.into())
        }
    }

    pub async fn activate<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        fingerprint: &String,
        components: &[Component],
    ) -> Result<Machine> {
        if let Some(license) = &self.license {
            let machine = license.activate(fingerprint, components).await?;
            Self::save_license_key_cache(app_handle, &license)?;
            Ok(machine)
        } else {
            Err(Error::NoLicenseError)
        }
    }

    pub async fn deactivate<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        fingerprint: &String,
    ) -> Result<()> {
        if let Some(license) = &self.license {
            license.deactivate(fingerprint).await?;
            Self::remove_license_file(app_handle)?;
            MachineState::remove_machine_file(app_handle)?;
            Ok(())
        } else {
            Err(Error::NoLicenseError)
        }
    }

    pub async fn checkout<R: Runtime>(
        &self,
        app_handle: &AppHandle<R>,
        options: &LicenseCheckoutOpts,
    ) -> Result<LicenseFile> {
        if let Some(license) = &self.license {
            let license_file = license.checkout(options).await?;
            Self::save_license_file(app_handle, &license_file)?;
            Ok(license_file)
        } else {
            Err(Error::NoLicenseError)
        }
    }

    pub(crate) fn load<R: Runtime>(app_handle: &AppHandle<R>) -> Result<Self> {
        if let Some(license_key) = Self::load_license_key_cache(app_handle)? {
            keygen_rs::config::set_license_key(&license_key);
            // attempt to load the license file
            if let Some(license_file) = Self::load_license_file(app_handle, &license_key)? {
                let dataset = license_file.decrypt(&license_key)?;
                return Ok(Self {
                    key: Some(license_key),
                    license: Some(dataset.license),
                });
            }
            // attempt to load the machine file
            let fingerprint = MachineState::get_fingerprint();
            let key = format!("{}{}", &license_key, fingerprint);
            if let Some(machine_file) = MachineState::load_machine_file(app_handle, &key)? {
                let dataset = machine_file.decrypt(&key)?;
                if dataset.license.key == license_key && dataset.machine.fingerprint == fingerprint
                {
                    return Ok(Self {
                        key: Some(license_key),
                        license: Some(dataset.license),
                    });
                }
            }
            return Ok(Self {
                key: Some(license_key),
                license: None,
            });
        }
        Ok(Self {
            key: None,
            license: None,
        })
    }

    fn get_license_key_cache_path<R: Runtime>(app_handle: &AppHandle<R>) -> Result<PathBuf> {
        let data_dir = get_app_keygen_path(app_handle)?;
        let path = data_dir.join("key");
        Ok(path)
    }

    fn load_license_key_cache<R: Runtime>(app_handle: &AppHandle<R>) -> Result<Option<String>> {
        let path = Self::get_license_key_cache_path(app_handle)?;
        if !path.exists() {
            return Ok(None);
        }
        let key = fs::read_to_string(path)?;
        Ok(Some(key))
    }

    fn save_license_key_cache<R: Runtime>(
        app_handle: &AppHandle<R>,
        license: &License,
    ) -> Result<()> {
        let path = Self::get_license_key_cache_path(app_handle)?;
        let mut file = File::create(&path)?;
        file.write_all(license.key.as_bytes())?;
        Ok(())
    }

    fn get_license_file_path<R: Runtime>(app_handle: &AppHandle<R>) -> Result<PathBuf> {
        let data_dir = get_app_keygen_path(app_handle)?;
        let path = data_dir.join("license.lic");
        Ok(path)
    }

    fn load_license_file<R: Runtime>(
        app_handle: &AppHandle<R>,
        key: &str,
    ) -> Result<Option<LicenseFile>> {
        let path = Self::get_license_file_path(app_handle)?;
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        let license_file = LicenseFile::from_cert(key, &content)?;
        Ok(Some(license_file))
    }

    fn save_license_file<R: Runtime>(
        app_handle: &AppHandle<R>,
        license_file: &LicenseFile,
    ) -> Result<()> {
        let path = Self::get_license_file_path(app_handle)?;
        let mut file = File::create(path)?;
        file.write_all(license_file.certificate.as_bytes())?;
        Ok(())
    }

    fn remove_license_file<R: Runtime>(app_handle: &AppHandle<R>) -> Result<()> {
        let path = Self::get_license_file_path(app_handle)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
