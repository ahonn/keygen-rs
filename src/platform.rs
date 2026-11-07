use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
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

/// Options for listing platforms
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListPlatformsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Cursor returned by [`Page::next_cursor`](crate::pagination::Page::next_cursor).
    /// Ignored when `page_number` is set.
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
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
        let page = Self::list_with_client(&Client::from_global_config()?, options.as_ref()).await?;
        Ok(page.into_items())
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListPlatformsOptions>,
    ) -> Result<Page<Platform>, Error> {
        let query = list_query(options)?;
        let response = client.get("platforms", query.as_ref()).await?;
        let document: ListDocument<PlatformAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Platform::from))
    }

    /// Get a platform by ID
    pub async fn get(id: &str) -> Result<Platform, Error> {
        Self::get_with_client(&Client::from_global_config()?, id).await
    }

    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Platform, Error> {
        let endpoint = format!("platforms/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let platform_response: PlatformResponse = serde_json::from_value(response.body)?;
        Ok(Platform::from(platform_response.data))
    }
}

/// Platform operations bound to one [`KeygenClient`].
pub struct PlatformService<'a> {
    client: &'a KeygenClient,
}

impl<'a> PlatformService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        options: Option<&ListPlatformsOptions>,
    ) -> Result<Page<Platform>, Error> {
        Platform::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<Platform, Error> {
        Platform::get_with_client(&self.client.transport, id).await
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
