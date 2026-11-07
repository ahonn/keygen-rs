use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};

/// Architecture attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchAttributes {
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArchResponse {
    pub data: KeygenResponseData<ArchAttributes>,
}

/// Options for listing architectures
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListArchesOptions {
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

/// An architecture represents a target CPU architecture for artifacts
///
/// Architectures are read-only and automatically populated by releases and artifacts.
/// Common architectures include: amd64, arm64, x86, arm
#[derive(Debug, Clone)]
pub struct Arch {
    pub id: String,
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl Arch {
    pub(crate) fn from(data: KeygenResponseData<ArchAttributes>) -> Arch {
        Arch {
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

    /// List all architectures with optional pagination
    ///
    /// Architectures are automatically populated based on releases and artifacts.
    pub async fn list(options: Option<ListArchesOptions>) -> Result<Vec<Arch>, Error> {
        let page = Self::list_with_client(&Client::from_global_config()?, options.as_ref()).await?;
        Ok(page.into_items())
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListArchesOptions>,
    ) -> Result<Page<Arch>, Error> {
        let query = list_query(options)?;
        let response = client.get("arches", query.as_ref()).await?;
        let document: ListDocument<ArchAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Arch::from))
    }

    /// Get an architecture by ID
    pub async fn get(id: &str) -> Result<Arch, Error> {
        Self::get_with_client(&Client::from_global_config()?, id).await
    }

    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Arch, Error> {
        let endpoint = format!("arches/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let arch_response: ArchResponse = serde_json::from_value(response.body)?;
        Ok(Arch::from(arch_response.data))
    }
}

/// Arch operations bound to one [`KeygenClient`].
pub struct ArchService<'a> {
    client: &'a KeygenClient,
}

impl<'a> ArchService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    pub async fn list(&self, options: Option<&ListArchesOptions>) -> Result<Page<Arch>, Error> {
        Arch::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<Arch, Error> {
        Arch::get_with_client(&self.client.transport, id).await
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
    fn test_arch_from_response_data() {
        let arch_data = KeygenResponseData {
            id: "test-arch-id".to_string(),
            r#type: "arches".to_string(),
            attributes: ArchAttributes {
                name: Some("AMD64".to_string()),
                key: "amd64".to_string(),
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

        let arch = Arch::from(arch_data);

        assert_eq!(arch.id, "test-arch-id");
        assert_eq!(arch.name, Some("AMD64".to_string()));
        assert_eq!(arch.key, "amd64");
        assert_eq!(arch.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_arch_without_name() {
        let arch_data = KeygenResponseData {
            id: "test-arch-id".to_string(),
            r#type: "arches".to_string(),
            attributes: ArchAttributes {
                name: None,
                key: "arm64".to_string(),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships::default(),
        };

        let arch = Arch::from(arch_data);

        assert_eq!(arch.id, "test-arch-id");
        assert_eq!(arch.name, None);
        assert_eq!(arch.key, "arm64");
    }
}
