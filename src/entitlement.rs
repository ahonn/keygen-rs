#[cfg(feature = "token")]
use crate::client::Client;
#[cfg(feature = "token")]
use crate::errors::Error;
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementAttributes {
    pub name: Option<String>,
    pub code: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EntitlementResponse {
    pub data: KeygenResponseData<EntitlementAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EntitlementsResponse {
    pub data: Vec<KeygenResponseData<EntitlementAttributes>>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntitlementRequest {
    pub name: Option<String>,
    pub code: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListEntitlementsOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntitlementRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub id: String,
    pub name: Option<String>,
    pub code: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
}

impl Entitlement {
    pub(crate) fn from(data: KeygenResponseData<EntitlementAttributes>) -> Entitlement {
        Entitlement {
            id: data.id,
            name: data.attributes.name,
            code: data.attributes.code,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new entitlement
    #[cfg(feature = "token")]
    pub async fn create(request: CreateEntitlementRequest) -> Result<Entitlement, Error> {
        let client = Client::default()?;

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        attributes.insert("code".to_string(), serde_json::Value::String(request.code));
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "entitlements",
                "attributes": attributes
            }
        });

        let response = client
            .post("entitlements", Some(&body), None::<&()>)
            .await?;
        let entitlement_response: EntitlementResponse = serde_json::from_value(response.body)?;
        Ok(Entitlement::from(entitlement_response.data))
    }

    /// List entitlements with optional pagination and filtering
    #[cfg(feature = "token")]
    pub async fn list(options: Option<ListEntitlementsOptions>) -> Result<Vec<Entitlement>, Error> {
        let client = Client::default()?;
        let response = client.get("entitlements", options.as_ref()).await?;
        let entitlements_response: EntitlementsResponse = serde_json::from_value(response.body)?;
        Ok(entitlements_response
            .data
            .into_iter()
            .map(Entitlement::from)
            .collect())
    }

    /// Get an entitlement by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<Entitlement, Error> {
        let client = Client::default()?;
        let endpoint = format!("entitlements/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let entitlement_response: EntitlementResponse = serde_json::from_value(response.body)?;
        Ok(Entitlement::from(entitlement_response.data))
    }

    /// Update an entitlement
    #[cfg(feature = "token")]
    pub async fn update(&self, request: UpdateEntitlementRequest) -> Result<Entitlement, Error> {
        let client = Client::default()?;
        let endpoint = format!("entitlements/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(code) = request.code {
            attributes.insert("code".to_string(), serde_json::Value::String(code));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "entitlements",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let entitlement_response: EntitlementResponse = serde_json::from_value(response.body)?;
        Ok(Entitlement::from(entitlement_response.data))
    }

    /// Delete an entitlement
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("entitlements/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }
}

#[cfg(all(test, feature = "token"))]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_entitlement_account_relationship() {
        let entitlement_data = KeygenResponseData {
            id: "test-entitlement-id".to_string(),
            r#type: "entitlements".to_string(),
            attributes: EntitlementAttributes {
                name: Some("Premium Feature".to_string()),
                code: "premium".to_string(),
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".parse().unwrap(),
                updated: "2023-01-01T00:00:00Z".parse().unwrap(),
            },
            relationships: KeygenRelationships {
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

        let entitlement = Entitlement::from(entitlement_data);

        assert_eq!(entitlement.account_id, Some("test-account-id".to_string()));
        assert_eq!(entitlement.id, "test-entitlement-id");
        assert_eq!(entitlement.name, Some("Premium Feature".to_string()));
        assert_eq!(entitlement.code, "premium");
    }

    #[test]
    fn test_entitlement_without_account_relationship() {
        let entitlement_data = KeygenResponseData {
            id: "test-entitlement-id".to_string(),
            r#type: "entitlements".to_string(),
            attributes: EntitlementAttributes {
                name: None,
                code: "basic".to_string(),
                metadata: None,
                created: "2023-01-01T00:00:00Z".parse().unwrap(),
                updated: "2023-01-01T00:00:00Z".parse().unwrap(),
            },
            relationships: KeygenRelationships::default(),
        };

        let entitlement = Entitlement::from(entitlement_data);

        assert_eq!(entitlement.account_id, None);
        assert_eq!(entitlement.name, None);
        assert_eq!(entitlement.metadata, None);
    }

    #[test]
    fn test_create_entitlement_request_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "feature_level".to_string(),
            serde_json::Value::String("premium".to_string()),
        );

        let request = CreateEntitlementRequest {
            name: Some("Advanced Features".to_string()),
            code: "advanced".to_string(),
            metadata: Some(metadata),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        assert!(serialized.contains("\"code\":\"advanced\""));
        assert!(serialized.contains("\"name\":\"Advanced Features\""));
        assert!(serialized.contains("\"metadata\""));
    }

    #[test]
    fn test_list_entitlements_options_serialization() {
        let options = ListEntitlementsOptions {
            limit: Some(10),
            page_size: Some(5),
            page_number: Some(2),
        };

        let serialized = serde_json::to_string(&options).unwrap();
        assert!(serialized.contains("\"limit\":10"));
        assert!(serialized.contains("\"page[size]\":5"));
        assert!(serialized.contains("\"page[number]\":2"));
    }
}
