use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Package engine type for distribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PackageEngine {
    Pypi,
    Tauri,
    Rubygems,
    Npm,
    Oci,
    Raw,
}

/// Package attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageAttributes {
    pub name: String,
    pub key: String,
    pub engine: Option<PackageEngine>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PackageResponse {
    pub data: KeygenResponseData<PackageAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct PackagesResponse {
    pub data: Vec<KeygenResponseData<PackageAttributes>>,
}

/// Request to create a new package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePackageRequest {
    /// Human-readable name of the package
    pub name: String,
    /// Machine-readable key of the package
    pub key: String,
    /// Associated product ID
    pub product_id: String,
    /// Optional: Engine for the package (pypi, tauri, rubygems, npm, oci, raw)
    pub engine: Option<PackageEngine>,
    /// Optional: Custom metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing package
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdatePackageRequest {
    pub name: Option<String>,
    pub key: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for listing packages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListPackagesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Filter by product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Filter by engine
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<PackageEngine>,
}

/// A package groups releases for distribution
#[derive(Debug, Clone)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub key: String,
    pub engine: Option<PackageEngine>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
}

impl Package {
    pub(crate) fn from(data: KeygenResponseData<PackageAttributes>) -> Package {
        Package {
            id: data.id,
            name: data.attributes.name,
            key: data.attributes.key,
            engine: data.attributes.engine,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
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
            environment_id: data
                .relationships
                .environment
                .as_ref()
                .and_then(|e| e.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new package
    pub async fn create(request: CreatePackageRequest) -> Result<Package, Error> {
        let client = Client::default()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert("name".to_string(), serde_json::Value::String(request.name));
        attributes.insert("key".to_string(), serde_json::Value::String(request.key));

        if let Some(engine) = request.engine {
            attributes.insert("engine".to_string(), serde_json::to_value(engine)?);
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "packages",
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

        let response = client.post("packages", Some(&body), None::<&()>).await?;
        let package_response: PackageResponse = serde_json::from_value(response.body)?;
        Ok(Package::from(package_response.data))
    }

    /// List packages with optional filtering and pagination
    pub async fn list(options: Option<ListPackagesOptions>) -> Result<Vec<Package>, Error> {
        let client = Client::default()?;
        let response = client.get("packages", options.as_ref()).await?;
        let packages_response: PackagesResponse = serde_json::from_value(response.body)?;
        Ok(packages_response
            .data
            .into_iter()
            .map(Package::from)
            .collect())
    }

    /// Get a package by ID or key
    pub async fn get(id: &str) -> Result<Package, Error> {
        let client = Client::default()?;
        let endpoint = format!("packages/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let package_response: PackageResponse = serde_json::from_value(response.body)?;
        Ok(Package::from(package_response.data))
    }

    /// Update an existing package
    pub async fn update(&self, request: UpdatePackageRequest) -> Result<Package, Error> {
        let client = Client::default()?;
        let endpoint = format!("packages/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(key) = request.key {
            attributes.insert("key".to_string(), serde_json::Value::String(key));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "packages",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let package_response: PackageResponse = serde_json::from_value(response.body)?;
        Ok(Package::from(package_response.data))
    }

    /// Delete a package
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("packages/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_package_from_response_data() {
        let package_data = KeygenResponseData {
            id: "test-package-id".to_string(),
            r#type: "packages".to_string(),
            attributes: PackageAttributes {
                name: "My Package".to_string(),
                key: "my-package".to_string(),
                engine: Some(PackageEngine::Raw),
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
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

        let package = Package::from(package_data);

        assert_eq!(package.id, "test-package-id");
        assert_eq!(package.name, "My Package");
        assert_eq!(package.key, "my-package");
        assert_eq!(package.engine, Some(PackageEngine::Raw));
        assert_eq!(package.product_id, Some("test-product-id".to_string()));
        assert_eq!(package.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_package_engine_serialization() {
        assert_eq!(
            serde_json::to_string(&PackageEngine::Pypi).unwrap(),
            "\"pypi\""
        );
        assert_eq!(
            serde_json::to_string(&PackageEngine::Tauri).unwrap(),
            "\"tauri\""
        );
        assert_eq!(
            serde_json::to_string(&PackageEngine::Npm).unwrap(),
            "\"npm\""
        );
        assert_eq!(
            serde_json::to_string(&PackageEngine::Raw).unwrap(),
            "\"raw\""
        );
    }
}
