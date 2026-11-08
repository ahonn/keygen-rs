use crate::client::Client;
use crate::errors::Error;
use crate::pagination::Pagination;
use crate::ApiContractVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct PingResponse {
    pub message: String,
    pub version: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ServiceInfo {
    /// Server timestamp
    pub timestamp: Option<String>,
    /// API version from response or headers
    pub api_version: Option<String>,
    /// Ping message
    pub message: Option<String>,
    /// Server headers
    pub headers: HashMap<String, String>,
}

/// The edition of a Keygen server, from the `Keygen-Edition` response header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[non_exhaustive]
pub enum KeygenEdition {
    /// Keygen CE, the self-hosted Community Edition.
    #[serde(rename = "CE")]
    Community,
    /// Keygen EE, the Enterprise Edition. Keygen Cloud also reports `EE`.
    #[serde(rename = "EE")]
    Enterprise,
}

impl KeygenEdition {
    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            v if v.eq_ignore_ascii_case("CE") => Some(Self::Community),
            v if v.eq_ignore_ascii_case("EE") => Some(Self::Enterprise),
            _ => None,
        }
    }
}

/// The tenancy mode of a Keygen server, from the `Keygen-Mode` response header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum KeygenMode {
    /// One account per server.
    Singleplayer,
    /// Many accounts per server, as on Keygen Cloud.
    Multiplayer,
}

impl KeygenMode {
    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            v if v.eq_ignore_ascii_case("singleplayer") => Some(Self::Singleplayer),
            v if v.eq_ignore_ascii_case("multiplayer") => Some(Self::Multiplayer),
            _ => None,
        }
    }
}

/// What a Keygen server supports, read from the headers of a ping response.
///
/// Keygen sends `Keygen-Edition` (`CE` or `EE`), `Keygen-Mode` (`singleplayer` or
/// `multiplayer`), `Keygen-Version` and `Keygen-Revision` on its API responses.
/// A field is `None` when the header is missing or has an unknown value, for example
/// on an old server or behind a proxy that removes headers.
///
/// The `supports_*` predicates use the edition only. They return `false` when the
/// edition is unknown. A self-hosted EE server can still refuse an EE feature when its
/// license does not include it, or when the feature needs a service (such as ClickHouse
/// for request and event logs) that the server does not have.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[non_exhaustive]
pub struct Capabilities {
    /// `Keygen-Edition`.
    pub edition: Option<KeygenEdition>,
    /// `Keygen-Mode`.
    pub mode: Option<KeygenMode>,
    /// `Keygen-Version`: the API version the server used for the ping request.
    pub api_version: Option<String>,
    /// `Keygen-Revision`: the server build.
    pub revision: Option<String>,
}

impl Capabilities {
    /// Reads the capability headers of a ping response.
    pub fn from_service_info(service_info: &ServiceInfo) -> Self {
        let header = |name: &str| {
            service_info
                .headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.trim())
                .filter(|value| !value.is_empty())
        };

        Self {
            edition: header("keygen-edition").and_then(KeygenEdition::parse),
            mode: header("keygen-mode").and_then(KeygenMode::parse),
            api_version: service_info
                .api_version
                .clone()
                .or_else(|| header("keygen-version").map(str::to_owned)),
            revision: header("keygen-revision").map(str::to_owned),
        }
    }

    /// Whether the server is Keygen EE (self-hosted EE or Keygen Cloud).
    pub fn is_enterprise(&self) -> bool {
        self.edition == Some(KeygenEdition::Enterprise)
    }

    /// Whether the server is Keygen CE.
    pub fn is_community(&self) -> bool {
        self.edition == Some(KeygenEdition::Community)
    }

    /// Whether the server hosts many accounts, as Keygen Cloud does.
    pub fn is_multiplayer(&self) -> bool {
        self.mode == Some(KeygenMode::Multiplayer)
    }

    /// Event logs (`/event-logs`) are EE only.
    pub fn supports_event_logs(&self) -> bool {
        self.is_enterprise()
    }

    /// Request logs (`/request-logs`) are EE only.
    pub fn supports_request_logs(&self) -> bool {
        self.is_enterprise()
    }

    /// Environments (`/environments`) are EE only.
    pub fn supports_environments(&self) -> bool {
        self.is_enterprise()
    }

    /// Token and user `permissions` are EE only. Keygen CE omits them.
    pub fn supports_permissions(&self) -> bool {
        self.is_enterprise()
    }
}

impl From<&ServiceInfo> for Capabilities {
    fn from(service_info: &ServiceInfo) -> Self {
        Self::from_service_info(service_info)
    }
}

/// What a Keygen server supports, as [`KeygenClient::negotiate`](crate::KeygenClient::negotiate)
/// found it. A negotiated client sends every request to match it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ServerProfile {
    /// The edition, mode, API version and revision headers of the server's ping.
    pub capabilities: Capabilities,
    /// The API version the client requests: the server's, or the client's when the
    /// client asked for an older one.
    pub api_version: ApiContractVersion,
    /// How the server pages its lists.
    pub pagination: Pagination,
}

/// Get service information using the /v1/ping endpoint
/// This can help determine the Keygen.sh service version and capabilities
pub async fn get_service_info() -> Result<ServiceInfo, Error> {
    let client = Client::from_global_config()?;

    get_service_info_with_client(&client).await
}

pub(crate) async fn get_service_info_with_client(client: &Client) -> Result<ServiceInfo, Error> {
    // Use the ping endpoint to get version information
    let response = client.get_text("ping").await?;

    let mut headers = HashMap::new();
    for (name, value) in response.headers.iter() {
        if let Ok(value_str) = value.to_str() {
            headers.insert(name.to_string(), value_str.to_string());
        }
    }

    // Extract API version from headers
    let api_version = headers
        .get("keygen-version")
        .or_else(|| headers.get("x-api-version"))
        .or_else(|| headers.get("api-version"))
        .cloned();

    // Extract server timestamp from headers
    let timestamp = headers
        .get("date")
        .or_else(|| headers.get("x-timestamp"))
        .cloned();

    // Use the ping response text as message
    let message = Some(response.body.trim().to_string());

    Ok(ServiceInfo {
        timestamp,
        api_version,
        message,
        headers,
    })
}

/// Check whether the observed service contract is at least the required contract.
pub fn supports_api_contract(
    service_info: &ServiceInfo,
    required_version: ApiContractVersion,
) -> bool {
    service_info
        .api_version
        .as_deref()
        .and_then(|version| ApiContractVersion::resolve_observed(version).ok())
        .is_some_and(|version| version >= required_version)
}

/// Ping the Keygen service and get basic information
pub async fn ping() -> Result<PingResponse, Error> {
    ping_with_client(&Client::from_global_config()?).await
}

pub(crate) async fn ping_with_client(client: &Client) -> Result<PingResponse, Error> {
    let response = client.get_text("ping").await?;

    // The ping endpoint returns plain text (usually "ok")
    // We'll extract version info from headers if available
    let message = response.body.trim().to_string();

    let version = response
        .headers
        .get("keygen-version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let timestamp = response
        .headers
        .get("date")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    Ok(PingResponse {
        message,
        version,
        timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_api_contract() {
        let current_service = ServiceInfo {
            timestamp: None,
            api_version: Some("1.8".to_string()),
            message: None,
            headers: HashMap::new(),
        };

        assert!(supports_api_contract(
            &current_service,
            ApiContractVersion::V1_7
        ));
        assert!(supports_api_contract(
            &current_service,
            ApiContractVersion::V1_8
        ));

        let future_minor_service = ServiceInfo {
            api_version: Some("1.9".to_string()),
            ..current_service
        };
        assert!(supports_api_contract(
            &future_minor_service,
            ApiContractVersion::V1_8
        ));
    }
}
