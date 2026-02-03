use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Release channel for distribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Rc,
    Beta,
    Alpha,
    Dev,
}

/// Release status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseStatus {
    Draft,
    Published,
    Yanked,
}

/// Semantic version components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

/// Release attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAttributes {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub semver: Option<Semver>,
    pub channel: ReleaseChannel,
    pub status: ReleaseStatus,
    pub tag: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    #[serde(rename = "yanked")]
    pub yanked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReleaseResponse {
    pub data: KeygenResponseData<ReleaseAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReleasesResponse {
    pub data: Vec<KeygenResponseData<ReleaseAttributes>>,
}

/// Request to create a new release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    /// Version string (semver format, without 'v' prefix)
    pub version: String,
    /// Release channel
    pub channel: ReleaseChannel,
    /// Associated product ID
    pub product_id: String,
    /// Optional: Human-readable name
    pub name: Option<String>,
    /// Optional: Description or release notes
    pub description: Option<String>,
    /// Optional: Initial status (defaults to DRAFT)
    pub status: Option<ReleaseStatus>,
    /// Optional: Unique tag for lookups
    pub tag: Option<String>,
    /// Optional: Custom metadata (e.g., checksums)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing release
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateReleaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub channel: Option<ReleaseChannel>,
    pub tag: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for listing releases
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListReleasesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Filter by channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<ReleaseChannel>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ReleaseStatus>,
    /// Filter by version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Filter by product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
}

/// A release represents a specific version of your software
#[derive(Debug, Clone)]
pub struct Release {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub semver: Option<Semver>,
    pub channel: ReleaseChannel,
    pub status: ReleaseStatus,
    pub tag: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
}

impl Release {
    pub(crate) fn from(data: KeygenResponseData<ReleaseAttributes>) -> Release {
        Release {
            id: data.id,
            name: data.attributes.name,
            description: data.attributes.description,
            version: data.attributes.version,
            semver: data.attributes.semver,
            channel: data.attributes.channel,
            status: data.attributes.status,
            tag: data.attributes.tag,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            yanked_at: data.attributes.yanked_at,
            product_id: data
                .relationships
                .product
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new release
    pub async fn create(request: CreateReleaseRequest) -> Result<Release, Error> {
        let client = Client::from_global_config()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert(
            "version".to_string(),
            serde_json::Value::String(request.version),
        );
        attributes.insert(
            "channel".to_string(),
            serde_json::to_value(&request.channel)?,
        );

        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(description) = request.description {
            attributes.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }
        if let Some(status) = request.status {
            attributes.insert("status".to_string(), serde_json::to_value(status)?);
        }
        if let Some(tag) = request.tag {
            attributes.insert("tag".to_string(), serde_json::Value::String(tag));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "releases",
                "attributes": attributes,
                "relationships": {
                    "product": {
                        "data": {
                            "type": "products",
                            "id": request.product_id
                        }
                    }
                }
            }
        });

        let response = client.post("releases", Some(&body), None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// List releases with optional filtering and pagination
    pub async fn list(options: Option<ListReleasesOptions>) -> Result<Vec<Release>, Error> {
        let client = Client::from_global_config()?;
        let response = client.get("releases", options.as_ref()).await?;
        let releases_response: ReleasesResponse = serde_json::from_value(response.body)?;
        Ok(releases_response
            .data
            .into_iter()
            .map(Release::from)
            .collect())
    }

    /// Get a release by ID
    pub async fn get(id: &str) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Update an existing release
    pub async fn update(&self, request: UpdateReleaseRequest) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(description) = request.description {
            attributes.insert(
                "description".to_string(),
                serde_json::Value::String(description),
            );
        }
        if let Some(channel) = request.channel {
            attributes.insert("channel".to_string(), serde_json::to_value(channel)?);
        }
        if let Some(tag) = request.tag {
            attributes.insert("tag".to_string(), serde_json::Value::String(tag));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "releases",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Delete a release
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Publish a release (DRAFT -> PUBLISHED)
    ///
    /// Makes the release visible to customers
    pub async fn publish(&self) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/actions/publish", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Yank a release (PUBLISHED -> YANKED)
    ///
    /// Makes the release unavailable for distribution
    pub async fn yank(&self) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/actions/yank", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_release_from_response_data() {
        let release_data = KeygenResponseData {
            id: "test-release-id".to_string(),
            r#type: "releases".to_string(),
            attributes: ReleaseAttributes {
                name: Some("v1.0.0".to_string()),
                description: Some("Initial release".to_string()),
                version: "1.0.0".to_string(),
                semver: Some(Semver {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: None,
                    build: None,
                }),
                channel: ReleaseChannel::Stable,
                status: ReleaseStatus::Published,
                tag: Some("v1.0.0".to_string()),
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships {
                product: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    }),
                    links: None,
                }),
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                ..Default::default()
            },
        };

        let release = Release::from(release_data);

        assert_eq!(release.id, "test-release-id");
        assert_eq!(release.version, "1.0.0");
        assert_eq!(release.channel, ReleaseChannel::Stable);
        assert_eq!(release.status, ReleaseStatus::Published);
        assert_eq!(release.product_id, Some("test-product-id".to_string()));
        assert_eq!(release.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_release_without_relationships() {
        let release_data = KeygenResponseData {
            id: "test-release-id".to_string(),
            r#type: "releases".to_string(),
            attributes: ReleaseAttributes {
                name: None,
                description: None,
                version: "1.0.0-beta.1".to_string(),
                semver: Some(Semver {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: Some("beta.1".to_string()),
                    build: None,
                }),
                channel: ReleaseChannel::Beta,
                status: ReleaseStatus::Draft,
                tag: None,
                metadata: None,
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships::default(),
        };

        let release = Release::from(release_data);

        assert_eq!(release.id, "test-release-id");
        assert_eq!(release.channel, ReleaseChannel::Beta);
        assert_eq!(release.status, ReleaseStatus::Draft);
        assert_eq!(release.product_id, None);
        assert_eq!(release.account_id, None);
    }

    #[test]
    fn test_release_channel_serialization() {
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Stable).unwrap(),
            "\"stable\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Rc).unwrap(),
            "\"rc\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Beta).unwrap(),
            "\"beta\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Alpha).unwrap(),
            "\"alpha\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Dev).unwrap(),
            "\"dev\""
        );
    }

    #[test]
    fn test_release_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ReleaseStatus::Draft).unwrap(),
            "\"DRAFT\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseStatus::Published).unwrap(),
            "\"PUBLISHED\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseStatus::Yanked).unwrap(),
            "\"YANKED\""
        );
    }

    #[test]
    fn test_list_releases_options_serialization() {
        let options = ListReleasesOptions {
            channel: Some(ReleaseChannel::Dev),
            limit: Some(20),
            ..Default::default()
        };

        let query = serde_urlencoded::to_string(&options).unwrap();
        println!("Query string: {}", query);
        assert!(query.contains("channel=dev"));
        assert!(query.contains("limit=20"));
        // Verify None values are not included
        assert!(!query.contains("page"));
        assert!(!query.contains("status"));
    }
}
