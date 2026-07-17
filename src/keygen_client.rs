use crate::api_version::ApiContractVersion;
use crate::client::{Client, ClientOptions};
use crate::config::{get_config, KeygenConfig};
use crate::errors::Error;
#[cfg(feature = "license-key")]
use crate::license::License;
use crate::license::LicenseService;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct KeygenClient {
    pub(crate) transport: Arc<Client>,
    config: Arc<KeygenConfig>,
}

impl KeygenClient {
    pub fn builder() -> KeygenClientBuilder {
        KeygenClientBuilder::default()
    }

    pub fn new(config: KeygenConfig) -> Result<Self, Error> {
        let transport = Arc::new(Client::new(ClientOptions::from(config.clone()))?);
        Ok(Self {
            transport,
            config: Arc::new(config),
        })
    }

    pub fn from_global_config() -> Result<Self, Error> {
        Self::new(get_config()?)
    }

    pub fn api_contract_version(&self) -> ApiContractVersion {
        self.config.api_version
    }

    pub fn config(&self) -> &KeygenConfig {
        &self.config
    }

    pub(crate) fn config_arc(&self) -> Arc<KeygenConfig> {
        Arc::clone(&self.config)
    }

    pub(crate) fn transport_arc(&self) -> Arc<Client> {
        Arc::clone(&self.transport)
    }

    pub fn licenses(&self) -> LicenseService<'_> {
        LicenseService::new(self)
    }

    #[cfg(feature = "license-key")]
    pub async fn validate(
        &self,
        fingerprints: &[String],
        entitlements: &[String],
    ) -> Result<License, Error> {
        self.licenses()
            .validate_key(
                self.config
                    .license_key
                    .as_deref()
                    .ok_or(Error::LicenseKeyMissing)?,
                fingerprints,
                entitlements,
            )
            .await
    }

    #[must_use = "verification result should be checked"]
    #[cfg(feature = "license-key")]
    pub fn verify(
        &self,
        scheme: crate::license::SchemeCode,
        signed_key: &str,
    ) -> Result<Vec<u8>, Error> {
        self.licenses().verify(scheme, signed_key)
    }
}

#[derive(Clone, Debug, Default)]
pub struct KeygenClientBuilder {
    config: KeygenConfig,
}

impl KeygenClientBuilder {
    pub fn account(mut self, account: impl Into<String>) -> Self {
        self.config.account = account.into();
        self
    }

    pub fn api_url(mut self, api_url: impl Into<String>) -> Self {
        self.config.api_url = api_url.into();
        self
    }

    pub fn api_contract_version(mut self, api_version: ApiContractVersion) -> Self {
        self.config.api_version = api_version;
        self
    }

    pub fn api_prefix(mut self, api_prefix: impl Into<String>) -> Self {
        self.config.api_prefix = api_prefix.into();
        self
    }

    pub fn environment(mut self, environment: impl Into<String>) -> Self {
        self.config.environment = Some(environment.into());
        self
    }

    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = Some(user_agent.into());
        self
    }

    #[cfg(feature = "license-key")]
    pub fn product(mut self, product: impl Into<String>) -> Self {
        self.config.product = product.into();
        self
    }

    #[cfg(feature = "license-key")]
    pub fn package(mut self, package: impl Into<String>) -> Self {
        self.config.package = package.into();
        self
    }

    #[cfg(feature = "license-key")]
    pub fn license_key(mut self, license_key: impl Into<String>) -> Self {
        self.config.license_key = Some(license_key.into());
        self
    }

    #[cfg(feature = "license-key")]
    pub fn public_key(mut self, public_key: impl Into<String>) -> Self {
        self.config.public_key = Some(public_key.into());
        self
    }

    #[cfg(feature = "license-key")]
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.config.platform = Some(platform.into());
        self
    }

    #[cfg(feature = "license-key")]
    pub fn max_clock_drift(mut self, max_clock_drift: i64) -> Self {
        self.config.max_clock_drift = Some(max_clock_drift);
        self
    }

    #[cfg(feature = "license-key")]
    pub fn verify_keygen_signature(mut self, verify: bool) -> Self {
        self.config.verify_keygen_signature = Some(verify);
        self
    }

    #[cfg(feature = "token")]
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.config.token = Some(token.into());
        self
    }

    pub fn build(self) -> Result<KeygenClient, Error> {
        KeygenClient::new(self.config)
    }
}
