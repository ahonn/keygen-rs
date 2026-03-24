use crate::artifact::{Artifact, ListArtifactsOptions};
use crate::client::Client;
use crate::config::get_config;
use crate::errors::Error;
use crate::insert_optional;
use crate::license::PaginationOptions;
use crate::KeygenRelationship;
use crate::KeygenResponseData;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{redirect::Policy, Client as ReqwestClient};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
use url::Url;

fn serialize_string_vec<S>(value: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(values) => values.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

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
    /// Filter by package ID or key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Filter by engine ID or key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Filter by entitlement codes
    #[serde(
        rename = "entitlements[]",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_string_vec"
    )]
    pub entitlements: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseUpgradeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<ReleaseChannel>,
}

#[derive(Debug, Clone)]
pub struct ReleaseArtifactDownload {
    pub location: String,
    pub headers: HeaderMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintAttributes {
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub entitlement_id: Option<String>,
    pub release_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConstraintsResponse {
    pub data: Vec<KeygenResponseData<ConstraintAttributes>>,
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
    pub package_id: Option<String>,
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
            package_id: data
                .relationships
                .other
                .get("package")
                .and_then(|value| serde_json::from_value::<KeygenRelationship>(value.clone()).ok())
                .and_then(|rel| rel.data.map(|d| d.id)),
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

        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "description", request.description)?;
        insert_optional(&mut attributes, "status", request.status)?;
        insert_optional(&mut attributes, "tag", request.tag)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

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

    /// Upgrade a release according to the provided constraints.
    pub async fn upgrade(&self, request: Option<&ReleaseUpgradeRequest>) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/upgrade", self.id);
        let response = client.get(&endpoint, request).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Update an existing release
    pub async fn update(&self, request: UpdateReleaseRequest) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}", self.id);

        let mut attributes = serde_json::Map::new();
        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "description", request.description)?;
        insert_optional(&mut attributes, "channel", request.channel)?;
        insert_optional(&mut attributes, "tag", request.tag)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

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

    /// Download an artifact by ID or filename, returning the redirect URL.
    pub async fn download_artifact(
        &self,
        artifact: &str,
    ) -> Result<ReleaseArtifactDownload, Error> {
        let config = get_config()?;
        let mut url = Url::parse(&config.api_url)?;
        url.path_segments_mut()
            .map_err(|_| Error::InvalidUrl)?
            .push(config.api_prefix.as_str())
            .push("accounts")
            .push(config.account.as_str())
            .push("releases")
            .push(self.id.as_str())
            .push("artifacts")
            .push(artifact);

        let client = ReqwestClient::builder()
            .redirect(Policy::none())
            .build()
            .map_err(|e| Error::UnexpectedError(format!("Failed to build HTTP client: {e}")))?;

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.api+json"));
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.api+json"),
        );
        headers.insert(
            "Keygen-Version",
            HeaderValue::from_str(&config.api_version)?,
        );
        if let Some(environment) = &config.environment {
            headers.insert("Keygen-Environment", HeaderValue::from_str(environment)?);
        }
        if let Some(user_agent) = &config.user_agent {
            headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)?);
        }
        if let Some(token) = &config.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))?,
            );
        } else if let Some(license_key) = &config.license_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("License {license_key}"))?,
            );
        }

        let response = client.get(url).headers(headers).send().await?;
        if response.status().is_client_error() || response.status().is_server_error() {
            let body = response.json().await?;
            return Err(Error::KeygenApiError {
                code: "DOWNLOAD_FAILED".to_string(),
                detail: "Failed to download release artifact".to_string(),
                body,
            });
        }

        let headers = response.headers().clone();
        let location = headers
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Error::UnexpectedError("Missing redirect Location header".to_string()))?
            .to_string();

        Ok(ReleaseArtifactDownload { location, headers })
    }

    /// List artifacts scoped to this release.
    pub async fn artifacts(
        &self,
        options: Option<ListArtifactsOptions>,
    ) -> Result<Vec<Artifact>, Error> {
        let mut options = options.unwrap_or_default();
        options.release = Some(self.id.clone());
        Artifact::list(Some(options)).await
    }

    /// Attach entitlement constraints to this release.
    pub async fn attach_constraints(
        &self,
        entitlement_ids: &[String],
    ) -> Result<Vec<Constraint>, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/constraints", self.id);
        let data: Vec<serde_json::Value> = entitlement_ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "type": "constraints",
                    "relationships": {
                        "entitlement": {
                            "data": {
                                "type": "entitlements",
                                "id": id
                            }
                        }
                    }
                })
            })
            .collect();
        let body = serde_json::json!({ "data": data });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let constraints_response: ConstraintsResponse = serde_json::from_value(response.body)?;
        Ok(constraints_response
            .data
            .into_iter()
            .map(Constraint::from)
            .collect())
    }

    /// Detach constraints from this release by constraint ID.
    pub async fn detach_constraints(&self, constraint_ids: &[String]) -> Result<(), Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/constraints", self.id);
        let data: Vec<serde_json::Value> = constraint_ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "type": "constraints",
                    "id": id
                })
            })
            .collect();
        let body = serde_json::json!({ "data": data });
        client
            .delete::<serde_json::Value, serde_json::Value>(&endpoint, Some(&body))
            .await?;
        Ok(())
    }

    /// List entitlement constraints for this release.
    pub async fn constraints(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<Constraint>, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/constraints", self.id);
        let response = client.get(&endpoint, options).await?;
        let constraints_response: ConstraintsResponse = serde_json::from_value(response.body)?;
        Ok(constraints_response
            .data
            .into_iter()
            .map(Constraint::from)
            .collect())
    }

    /// Change the package associated with this release.
    pub async fn change_package(&self, package_id: &str) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/package", self.id);
        let body = serde_json::json!({
            "data": {
                "type": "packages",
                "id": package_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }
}

impl Constraint {
    fn from(data: KeygenResponseData<ConstraintAttributes>) -> Self {
        let entitlement = data
            .relationships
            .other
            .get("entitlement")
            .and_then(|value| serde_json::from_value::<KeygenRelationship>(value.clone()).ok())
            .and_then(|relationship| relationship.data.map(|d| d.id));

        Self {
            id: data.id,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data.relationships.account_id(),
            entitlement_id: entitlement,
            release_id: data
                .relationships
                .release
                .as_ref()
                .and_then(|rel| rel.data.as_ref().map(|d| d.id.clone())),
        }
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
