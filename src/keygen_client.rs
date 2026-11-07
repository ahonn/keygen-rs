use crate::api_version::ApiContractVersion;
use crate::client::{Client, ClientOptions};
use crate::config::{get_config, KeygenConfig};
use crate::errors::Error;
use crate::license::LicenseService;
#[cfg(feature = "license-key")]
use crate::license::{LicenseValidationRequest, LicenseValidationResult};
#[cfg(feature = "token")]
use crate::profile::CurrentProfile;
use crate::rate_limit::RateLimit;
use crate::release::{Release, ReleaseUpgradeRequest};
use crate::service::{Capabilities, ServiceInfo};
#[cfg(not(target_arch = "wasm32"))]
use crate::{artifact::Artifact, release::ArtifactDownloadOptions};
use std::sync::Arc;

/// A Keygen API client with its own configuration.
///
/// Every request made through this client or one of its services (for example
/// [`KeygenClient::products`] or [`KeygenClient::machines`]) uses this client's account,
/// credentials, API version and `Keygen-Environment` header, never the global
/// configuration. Several clients can be used at the same time in one process.
///
/// Licenses and machines returned by a service keep the client, so follow-up calls on
/// them (for example [`Machine::deactivate`](crate::machine::Machine::deactivate)) use the
/// same account. Other resource types do not keep a client: their instance methods
/// (for example `Product::delete`) use the global configuration. Use the service methods
/// that take an ID (for example `client.products().delete(id)`) for those resources.
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

    #[cfg(feature = "token")]
    pub fn products(&self) -> crate::product::ProductService<'_> {
        crate::product::ProductService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn policies(&self) -> crate::policy::PolicyService<'_> {
        crate::policy::PolicyService::new(self)
    }

    pub fn machines(&self) -> crate::machine::MachineService<'_> {
        crate::machine::MachineService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn users(&self) -> crate::user::UserService<'_> {
        crate::user::UserService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn groups(&self) -> crate::group::GroupService<'_> {
        crate::group::GroupService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn entitlements(&self) -> crate::entitlement::EntitlementService<'_> {
        crate::entitlement::EntitlementService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn tokens(&self) -> crate::token::TokenService<'_> {
        crate::token::TokenService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn environments(&self) -> crate::environment::EnvironmentService<'_> {
        crate::environment::EnvironmentService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn components(&self) -> crate::component::ComponentService<'_> {
        crate::component::ComponentService::new(self)
    }

    pub fn releases(&self) -> crate::release::ReleaseService<'_> {
        crate::release::ReleaseService::new(self)
    }

    pub fn packages(&self) -> crate::package::PackageService<'_> {
        crate::package::PackageService::new(self)
    }

    pub fn artifacts(&self) -> crate::artifact::ArtifactService<'_> {
        crate::artifact::ArtifactService::new(self)
    }

    pub fn platforms(&self) -> crate::platform::PlatformService<'_> {
        crate::platform::PlatformService::new(self)
    }

    pub fn arches(&self) -> crate::arch::ArchService<'_> {
        crate::arch::ArchService::new(self)
    }

    pub fn channels(&self) -> crate::channel::ChannelService<'_> {
        crate::channel::ChannelService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn webhook_endpoints(&self) -> crate::webhook::endpoint::WebhookEndpointService<'_> {
        crate::webhook::endpoint::WebhookEndpointService::new(self)
    }

    #[cfg(feature = "token")]
    pub fn webhook_events(&self) -> crate::webhook::event::WebhookEventService<'_> {
        crate::webhook::event::WebhookEventService::new(self)
    }

    /// Process operations. See [`crate::process::ProcessService`].
    pub fn processes(&self) -> crate::process::ProcessService<'_> {
        crate::process::ProcessService::new(self)
    }

    /// Second factor (2FA) operations for users.
    #[cfg(feature = "token")]
    pub fn second_factors(&self) -> crate::second_factor::SecondFactorService<'_> {
        crate::second_factor::SecondFactorService::new(self)
    }

    /// Event logs. Keygen EE and Keygen Cloud only.
    #[cfg(feature = "token")]
    pub fn event_logs(&self) -> crate::event_log::EventLogService<'_> {
        crate::event_log::EventLogService::new(self)
    }

    /// Request logs. Keygen EE and Keygen Cloud only.
    #[cfg(feature = "token")]
    pub fn request_logs(&self) -> crate::request_log::RequestLogService<'_> {
        crate::request_log::RequestLogService::new(self)
    }

    /// Search through Keygen's undocumented search endpoint. See [`crate::search`].
    ///
    /// Requires the `unstable-search` feature. The endpoint is not part of Keygen's
    /// public API and can change without notice.
    #[cfg(all(feature = "token", feature = "unstable-search"))]
    pub fn search(&self) -> crate::search::SearchService<'_> {
        crate::search::SearchService::new(self)
    }

    /// Reads the edition, mode and API version of the server from a ping response.
    /// See [`Capabilities`].
    pub async fn capabilities(&self) -> Result<Capabilities, Error> {
        Ok(Capabilities::from_service_info(&self.service_info().await?))
    }

    /// The rate limit headers of the most recent response this client received that
    /// carried them, or `None` before the first such response.
    ///
    /// Requests that run at the same time overwrite each other's status, so treat the
    /// value as approximate. After a `429` response, prefer
    /// [`Error::rate_limit`](crate::errors::Error::rate_limit).
    pub fn last_rate_limit(&self) -> Option<RateLimit> {
        self.transport.last_rate_limit()
    }

    pub async fn service_info(&self) -> Result<ServiceInfo, Error> {
        crate::service::get_service_info_with_client(&self.transport).await
    }

    /// Ping the Keygen service. See [`crate::service::ping`].
    pub async fn ping(&self) -> Result<crate::service::PingResponse, Error> {
        crate::service::ping_with_client(&self.transport).await
    }

    /// Check whether an upgrade is available. See [`Release::upgrade_from`].
    pub async fn upgrade_from(
        &self,
        current_version: &str,
        request: Option<&ReleaseUpgradeRequest>,
    ) -> Result<Option<Release>, Error> {
        crate::release::upgrade_from_with_client(
            &self.transport,
            &self.config,
            current_version,
            request,
        )
        .await
    }

    /// Download a release artifact to `path` and verify it. See [`Release::download_artifact_to`].
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn download_artifact_to(
        &self,
        release: &Release,
        artifact: &str,
        path: impl AsRef<std::path::Path>,
        options: Option<ArtifactDownloadOptions>,
    ) -> Result<Artifact, Error> {
        crate::release::download_with_client(
            &self.transport,
            &self.config,
            release,
            artifact,
            path.as_ref(),
            options,
        )
        .await
    }

    /// Retrieves the bearer and token used to authenticate this client.
    #[cfg(feature = "token")]
    pub async fn current_profile(&self) -> Result<CurrentProfile, Error> {
        crate::profile::get_current_profile_with_client(&self.transport).await
    }

    #[cfg(feature = "license-key")]
    pub async fn validate(
        &self,
        request: &LicenseValidationRequest,
    ) -> Result<LicenseValidationResult, Error> {
        let mut request = request.clone();
        if request.scope.product.is_none() {
            request.scope.product = Some(self.config.product.clone());
        }
        self.licenses()
            .validate_key(
                self.config
                    .license_key
                    .as_deref()
                    .ok_or(Error::LicenseKeyMissing)?,
                &request,
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
