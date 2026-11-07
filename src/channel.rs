use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};

/// Channel attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAttributes {
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ChannelResponse {
    pub data: KeygenResponseData<ChannelAttributes>,
}

/// Options for listing channels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListChannelsOptions {
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

/// A channel represents a release track for software distribution
///
/// Channels are read-only and automatically populated by releases.
/// Common channels include: stable, rc, beta, alpha, dev
#[derive(Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl Channel {
    pub(crate) fn from(data: KeygenResponseData<ChannelAttributes>) -> Channel {
        Channel {
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

    /// List all channels with optional pagination
    ///
    /// Channels are automatically populated based on releases.
    pub async fn list(options: Option<ListChannelsOptions>) -> Result<Vec<Channel>, Error> {
        let page = Self::list_with_client(&Client::from_global_config()?, options.as_ref()).await?;
        Ok(page.into_items())
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListChannelsOptions>,
    ) -> Result<Page<Channel>, Error> {
        let query = list_query(options)?;
        let response = client.get("channels", query.as_ref()).await?;
        let document: ListDocument<ChannelAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Channel::from))
    }

    /// Get a channel by ID
    pub async fn get(id: &str) -> Result<Channel, Error> {
        Self::get_with_client(&Client::from_global_config()?, id).await
    }

    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Channel, Error> {
        let endpoint = format!("channels/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let channel_response: ChannelResponse = serde_json::from_value(response.body)?;
        Ok(Channel::from(channel_response.data))
    }
}

/// Channel operations bound to one [`KeygenClient`].
pub struct ChannelService<'a> {
    client: &'a KeygenClient,
}

impl<'a> ChannelService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        options: Option<&ListChannelsOptions>,
    ) -> Result<Page<Channel>, Error> {
        Channel::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<Channel, Error> {
        Channel::get_with_client(&self.client.transport, id).await
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
    fn test_channel_from_response_data() {
        let channel_data = KeygenResponseData {
            id: "test-channel-id".to_string(),
            r#type: "channels".to_string(),
            attributes: ChannelAttributes {
                name: Some("Stable".to_string()),
                key: "stable".to_string(),
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

        let channel = Channel::from(channel_data);

        assert_eq!(channel.id, "test-channel-id");
        assert_eq!(channel.name, Some("Stable".to_string()));
        assert_eq!(channel.key, "stable");
        assert_eq!(channel.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_channel_without_name() {
        let channel_data = KeygenResponseData {
            id: "test-channel-id".to_string(),
            r#type: "channels".to_string(),
            attributes: ChannelAttributes {
                name: None,
                key: "beta".to_string(),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships::default(),
        };

        let channel = Channel::from(channel_data);

        assert_eq!(channel.id, "test-channel-id");
        assert_eq!(channel.name, None);
        assert_eq!(channel.key, "beta");
    }
}
