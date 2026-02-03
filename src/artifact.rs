use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Artifact status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactStatus {
    Waiting,
    Uploaded,
    Failed,
    Yanked,
}

/// Artifact attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactAttributes {
    pub filename: String,
    pub filetype: Option<String>,
    pub filesize: Option<u64>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: ArtifactStatus,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    #[serde(rename = "yanked")]
    pub yanked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArtifactResponse {
    pub data: KeygenResponseData<ArtifactAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArtifactsResponse {
    pub data: Vec<KeygenResponseData<ArtifactAttributes>>,
}

/// Request to create a new artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArtifactRequest {
    /// The filename of the artifact
    pub filename: String,
    /// Associated release ID
    pub release_id: String,
    /// Optional: File type/extension
    pub filetype: Option<String>,
    /// Optional: File size in bytes
    pub filesize: Option<u64>,
    /// Optional: Target platform (e.g., "darwin", "linux", "win32")
    pub platform: Option<String>,
    /// Optional: Target architecture (e.g., "amd64", "arm64")
    pub arch: Option<String>,
    /// Optional: Signature for verification
    pub signature: Option<String>,
    /// Optional: Checksum (e.g., SHA-256)
    pub checksum: Option<String>,
    /// Optional: Custom metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing artifact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateArtifactRequest {
    pub filename: Option<String>,
    pub filetype: Option<String>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for listing artifacts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListArtifactsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Filter by release ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    /// Filter by product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Filter by channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Filter by platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Filter by architecture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// Filter by filetype
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filetype: Option<String>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArtifactStatus>,
}

/// An artifact represents a distributable file associated with a release
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub filename: String,
    pub filetype: Option<String>,
    pub filesize: Option<u64>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: ArtifactStatus,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub release_id: Option<String>,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
}

impl Artifact {
    pub(crate) fn from(data: KeygenResponseData<ArtifactAttributes>) -> Artifact {
        Artifact {
            id: data.id,
            filename: data.attributes.filename,
            filetype: data.attributes.filetype,
            filesize: data.attributes.filesize,
            platform: data.attributes.platform,
            arch: data.attributes.arch,
            signature: data.attributes.signature,
            checksum: data.attributes.checksum,
            status: data.attributes.status,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            yanked_at: data.attributes.yanked_at,
            release_id: data
                .relationships
                .release
                .as_ref()
                .and_then(|r| r.data.as_ref().map(|d| d.id.clone())),
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

    /// Create a new artifact
    ///
    /// Note: After creating an artifact, you need to upload the actual file
    /// using the upload URL provided in the response links.
    pub async fn create(request: CreateArtifactRequest) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert(
            "filename".to_string(),
            serde_json::Value::String(request.filename),
        );

        if let Some(filetype) = request.filetype {
            attributes.insert("filetype".to_string(), serde_json::Value::String(filetype));
        }
        if let Some(filesize) = request.filesize {
            attributes.insert("filesize".to_string(), serde_json::json!(filesize));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), serde_json::Value::String(platform));
        }
        if let Some(arch) = request.arch {
            attributes.insert("arch".to_string(), serde_json::Value::String(arch));
        }
        if let Some(signature) = request.signature {
            attributes.insert(
                "signature".to_string(),
                serde_json::Value::String(signature),
            );
        }
        if let Some(checksum) = request.checksum {
            attributes.insert("checksum".to_string(), serde_json::Value::String(checksum));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "artifacts",
                "attributes": attributes,
                "relationships": {
                    "release": {
                        "data": {
                            "type": "releases",
                            "id": request.release_id
                        }
                    }
                }
            }
        });

        let response = client.post("artifacts", Some(&body), None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// List artifacts with optional filtering and pagination
    pub async fn list(options: Option<ListArtifactsOptions>) -> Result<Vec<Artifact>, Error> {
        let client = Client::from_global_config()?;
        let response = client.get("artifacts", options.as_ref()).await?;
        let artifacts_response: ArtifactsResponse = serde_json::from_value(response.body)?;
        Ok(artifacts_response
            .data
            .into_iter()
            .map(Artifact::from)
            .collect())
    }

    /// Get an artifact by ID
    pub async fn get(id: &str) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// Update an existing artifact
    pub async fn update(&self, request: UpdateArtifactRequest) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(filename) = request.filename {
            attributes.insert("filename".to_string(), serde_json::Value::String(filename));
        }
        if let Some(filetype) = request.filetype {
            attributes.insert("filetype".to_string(), serde_json::Value::String(filetype));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), serde_json::Value::String(platform));
        }
        if let Some(arch) = request.arch {
            attributes.insert("arch".to_string(), serde_json::Value::String(arch));
        }
        if let Some(signature) = request.signature {
            attributes.insert(
                "signature".to_string(),
                serde_json::Value::String(signature),
            );
        }
        if let Some(checksum) = request.checksum {
            attributes.insert("checksum".to_string(), serde_json::Value::String(checksum));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "artifacts",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// Delete an artifact
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Yank an artifact (make it unavailable for download)
    pub async fn yank(&self) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("artifacts/{}/actions/yank", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_artifact_from_response_data() {
        let artifact_data = KeygenResponseData {
            id: "test-artifact-id".to_string(),
            r#type: "artifacts".to_string(),
            attributes: ArtifactAttributes {
                filename: "app-1.0.0.dmg".to_string(),
                filetype: Some("dmg".to_string()),
                filesize: Some(10485760),
                platform: Some("darwin".to_string()),
                arch: Some("amd64".to_string()),
                signature: None,
                checksum: Some("abc123".to_string()),
                status: ArtifactStatus::Uploaded,
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships {
                release: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "releases".to_string(),
                        id: "test-release-id".to_string(),
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

        let artifact = Artifact::from(artifact_data);

        assert_eq!(artifact.id, "test-artifact-id");
        assert_eq!(artifact.filename, "app-1.0.0.dmg");
        assert_eq!(artifact.filetype, Some("dmg".to_string()));
        assert_eq!(artifact.filesize, Some(10485760));
        assert_eq!(artifact.platform, Some("darwin".to_string()));
        assert_eq!(artifact.arch, Some("amd64".to_string()));
        assert_eq!(artifact.status, ArtifactStatus::Uploaded);
        assert_eq!(artifact.release_id, Some("test-release-id".to_string()));
        assert_eq!(artifact.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_artifact_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Waiting).unwrap(),
            "\"WAITING\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Uploaded).unwrap(),
            "\"UPLOADED\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Failed).unwrap(),
            "\"FAILED\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Yanked).unwrap(),
            "\"YANKED\""
        );
    }
}
