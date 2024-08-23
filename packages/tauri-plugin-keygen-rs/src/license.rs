use std::{fs, path::PathBuf};

use crate::{error::Error, Result};
use keygen_rs::{component::Component, license::License, machine::Machine};
use tauri::{AppHandle, Runtime};

#[derive(Debug, Clone, Default)]
pub struct LicenseState {
    pub(crate) license: Option<License>,
}

impl LicenseState {
    pub fn get_license(&self) -> Option<License> {
        self.license.clone()
    }

    pub async fn validate_key(&mut self, key: &str, fingerprints: &[String]) -> Result<License> {
        keygen_rs::config::set_license_key(key);
        let license = keygen_rs::validate(fingerprints)
            .await
            .map_err(|e| Error::KeygenError(e))?;
        self.license = Some(license.clone());
        Ok(license)
    }

    pub async fn activate_machine(
        &self,
        fingerprint: &String,
        components: &[Component],
    ) -> Result<Machine> {
        if let Some(license) = &self.license {
            let machine = license.activate(fingerprint, components).await?;
            Ok(machine)
        } else {
            Err(Error::NoLicenseError)
        }
    }

    pub async fn deactivate_machine(&self, fingerprint: &String) -> Result<()> {
        if let Some(license) = &self.license {
            license.deactivate(fingerprint).await?;
            Ok(())
        } else {
            Err(Error::NoLicenseError)
        }
    }

    pub(crate) fn load<R: Runtime>(app: &AppHandle<R>) -> Result<Self> {
        if let Some(license_key) = Self::get_cached_license_key(app)? {
            keygen_rs::config::set_license_key(&license_key);
            Ok(Self { license: None })
        } else {
            Ok(Self { license: None })
        }
    }

    pub(crate) fn get_cached_license_key<R: Runtime>(app: &AppHandle<R>) -> Result<Option<String>> {
        let path = Self::get_license_key_cache_path(app)?;
        if !path.exists() {
            return Ok(None);
        }
        let key = fs::read_to_string(path)?;
        Ok(Some(key))
    }

    // pub(crate) fn remove_cached_license_key<R: Runtime>(app: &AppHandle<R>) -> Result<()> {
    //     let path = Self::get_license_key_cache_path(app)?;
    //     if path.exists() {
    //         fs::remove_file(&path).map_err(|e| Error::IoError(e))?;
    //     }
    //     Ok(())
    // }

    fn get_license_key_cache_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf> {
        let data_dir = app
            .path_resolver()
            .app_data_dir()
            .ok_or_else(|| Error::PathResolveError("Can't resolve app data dir".into()))?;

        let keygen_cache_dir = data_dir.join("keygen");

        if !keygen_cache_dir.exists() {
            fs::create_dir_all(&keygen_cache_dir)?;
        }

        let dir_path = keygen_cache_dir.join("key");

        Ok(dir_path)
    }
}
