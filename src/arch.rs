use crate::client::Client;
use crate::errors::Error;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArchesResponse {
    pub data: Vec<KeygenResponseData<ArchAttributes>>,
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
        let client = Client::from_global_config()?;
        let response = client.get("arches", options.as_ref()).await?;
        let arches_response: ArchesResponse = serde_json::from_value(response.body)?;
        Ok(arches_response.data.into_iter().map(Arch::from).collect())
    }

    /// Get an architecture by ID
    pub async fn get(id: &str) -> Result<Arch, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("arches/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let arch_response: ArchResponse = serde_json::from_value(response.body)?;
        Ok(Arch::from(arch_response.data))
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
