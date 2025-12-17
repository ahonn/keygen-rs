use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};

/// Platform attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformAttributes {
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformResponse {
    pub data: KeygenResponseData<PlatformAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PlatformsResponse {
    pub data: Vec<KeygenResponseData<PlatformAttributes>>,
}

/// Options for listing platforms
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListPlatformsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
}

/// A platform represents a target operating system for artifacts
///
/// Platforms are read-only and automatically populated by releases and artifacts.
/// Common platforms include: darwin (macOS), linux, win32 (Windows)
#[derive(Debug, Clone)]
pub struct Platform {
    pub id: String,
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl Platform {
    pub(crate) fn from(data: KeygenResponseData<PlatformAttributes>) -> Platform {
        Platform {
            id: data.id,
            name: data.attributes.name,
            key: data.attributes.key,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// List all platforms with optional pagination
    ///
    /// Platforms are automatically populated based on releases and artifacts.
    pub async fn list(options: Option<ListPlatformsOptions>) -> Result<Vec<Platform>, Error> {
        let client = Client::default()?;
        let response = client.get("platforms", options.as_ref()).await?;
        let platforms_response: PlatformsResponse = serde_json::from_value(response.body)?;
        Ok(platforms_response
            .data
            .into_iter()
            .map(Platform::from)
            .collect())
    }

    /// Get a platform by ID
    pub async fn get(id: &str) -> Result<Platform, Error> {
        let client = Client::default()?;
        let endpoint = format!("platforms/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let platform_response: PlatformResponse = serde_json::from_value(response.body)?;
        Ok(Platform::from(platform_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };
    use std::collections::HashMap;

    #[test]
    fn test_platform_from_response_data() {
        let platform_data = KeygenResponseData {
            id: "test-platform-id".to_string(),
            r#type: "platforms".to_string(),
            attributes: PlatformAttributes {
                name: Some("macOS".to_string()),
                key: "darwin".to_string(),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships {
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                policy: None,
                product: None,
                group: None,
                owner: None,
                users: None,
                machines: None,
                environment: None,
                license: None,
                release: None,
                other: HashMap::new(),
            },
        };

        let platform = Platform::from(platform_data);

        assert_eq!(platform.id, "test-platform-id");
        assert_eq!(platform.name, Some("macOS".to_string()));
        assert_eq!(platform.key, "darwin");
        assert_eq!(platform.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_platform_without_name() {
        let platform_data = KeygenResponseData {
            id: "test-platform-id".to_string(),
            r#type: "platforms".to_string(),
            attributes: PlatformAttributes {
                name: None,
                key: "linux".to_string(),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships::default(),
        };

        let platform = Platform::from(platform_data);

        assert_eq!(platform.id, "test-platform-id");
        assert_eq!(platform.name, None);
        assert_eq!(platform.key, "linux");
    }
}
