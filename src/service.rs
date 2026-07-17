use crate::client::Client;
use crate::errors::Error;
use crate::ApiContractVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub message: String,
    pub version: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let client = Client::from_global_config()?;
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
