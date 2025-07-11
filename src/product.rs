use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DistributionStrategy {
    Open,
    Closed,
    Licensed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    #[serde(rename = "macOS")]
    MacOs,
    Linux,
    #[serde(rename = "darwin")]
    Darwin,
    Android,
    #[serde(rename = "iOS")]
    Ios,
    Web,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    #[serde(rename = "license.read")]
    LicenseRead,
    #[serde(rename = "license.create")]
    LicenseCreate,
    #[serde(rename = "license.update")]
    LicenseUpdate,
    #[serde(rename = "license.delete")]
    LicenseDelete,
    #[serde(rename = "license.validate")]
    LicenseValidate,
    #[serde(rename = "machine.read")]
    MachineRead,
    #[serde(rename = "machine.create")]
    MachineCreate,
    #[serde(rename = "machine.update")]
    MachineUpdate,
    #[serde(rename = "machine.delete")]
    MachineDelete,
    #[serde(rename = "user.read")]
    UserRead,
    #[serde(rename = "user.create")]
    UserCreate,
    #[serde(rename = "user.update")]
    UserUpdate,
    #[serde(rename = "user.delete")]
    UserDelete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAttributes {
    pub name: String,
    pub code: Option<String>,
    #[serde(rename = "distributionStrategy")]
    pub distribution_strategy: Option<DistributionStrategy>,
    pub url: Option<String>,
    pub platforms: Option<Vec<Platform>>,
    pub permissions: Option<Vec<Permission>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProductResponse {
    pub data: KeygenResponseData<ProductAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProductsResponse {
    pub data: Vec<KeygenResponseData<ProductAttributes>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub code: String,
    #[serde(rename = "distributionStrategy")]
    pub distribution_strategy: Option<DistributionStrategy>,
    pub url: Option<String>,
    pub platforms: Option<Vec<Platform>>,
    pub permissions: Option<Vec<Permission>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListProductsOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    #[serde(rename = "distributionStrategy")]
    pub distribution_strategy: Option<DistributionStrategy>,
    pub url: Option<String>,
    pub platforms: Option<Vec<Platform>>,
    pub permissions: Option<Vec<Permission>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub code: Option<String>,
    pub distribution_strategy: Option<DistributionStrategy>,
    pub url: Option<String>,
    pub platforms: Option<Vec<Platform>>,
    pub permissions: Option<Vec<Permission>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl Product {
    pub(crate) fn from(data: KeygenResponseData<ProductAttributes>) -> Product {
        Product {
            id: data.id,
            name: data.attributes.name,
            code: data.attributes.code,
            distribution_strategy: data.attributes.distribution_strategy,
            url: data.attributes.url,
            platforms: data.attributes.platforms,
            permissions: data.attributes.permissions,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data.relationships.account.as_ref().and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new product
    pub async fn create(request: CreateProductRequest) -> Result<Product, Error> {
        let client = Client::default();

        let mut attributes = serde_json::Map::new();
        attributes.insert("name".to_string(), serde_json::Value::String(request.name));
        attributes.insert("code".to_string(), serde_json::Value::String(request.code));

        if let Some(distribution_strategy) = request.distribution_strategy {
            attributes.insert(
                "distributionStrategy".to_string(),
                serde_json::to_value(distribution_strategy)?,
            );
        }
        if let Some(url) = request.url {
            attributes.insert("url".to_string(), serde_json::Value::String(url));
        }
        if let Some(platforms) = request.platforms {
            attributes.insert("platforms".to_string(), serde_json::to_value(platforms)?);
        }
        if let Some(permissions) = request.permissions {
            attributes.insert(
                "permissions".to_string(),
                serde_json::to_value(permissions)?,
            );
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "products",
                "attributes": attributes
            }
        });

        let response = client.post("products", Some(&body), None::<&()>).await?;
        let product_response: ProductResponse = serde_json::from_value(response.body)?;
        Ok(Product::from(product_response.data))
    }

    /// List products with optional pagination and filtering
    pub async fn list(options: Option<ListProductsOptions>) -> Result<Vec<Product>, Error> {
        let client = Client::default();
        let response = client.get("products", options.as_ref()).await?;
        let products_response: ProductsResponse = serde_json::from_value(response.body)?;
        Ok(products_response
            .data
            .into_iter()
            .map(Product::from)
            .collect())
    }

    /// Get a product by ID
    pub async fn get(id: &str) -> Result<Product, Error> {
        let client = Client::default();
        let endpoint = format!("products/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let product_response: ProductResponse = serde_json::from_value(response.body)?;
        Ok(Product::from(product_response.data))
    }

    /// Update a product
    pub async fn update(&self, request: UpdateProductRequest) -> Result<Product, Error> {
        let client = Client::default();
        let endpoint = format!("products/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), serde_json::Value::String(name));
        }
        if let Some(code) = request.code {
            attributes.insert("code".to_string(), serde_json::Value::String(code));
        }
        if let Some(distribution_strategy) = request.distribution_strategy {
            attributes.insert(
                "distributionStrategy".to_string(),
                serde_json::to_value(distribution_strategy)?,
            );
        }
        if let Some(url) = request.url {
            attributes.insert("url".to_string(), serde_json::Value::String(url));
        }
        if let Some(platforms) = request.platforms {
            attributes.insert("platforms".to_string(), serde_json::to_value(platforms)?);
        }
        if let Some(permissions) = request.permissions {
            attributes.insert(
                "permissions".to_string(),
                serde_json::to_value(permissions)?,
            );
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "products",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let product_response: ProductResponse = serde_json::from_value(response.body)?;
        Ok(Product::from(product_response.data))
    }

    /// Delete a product
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default();
        let endpoint = format!("products/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Generate a product token
    pub async fn generate_token(&self) -> Result<String, Error> {
        let client = Client::default();
        let endpoint = format!("products/{}/tokens", self.id);
        let response: crate::client::Response<serde_json::Value> =
            client.post(&endpoint, None::<&()>, None::<&()>).await?;

        // Extract token from response
        let token_data = response.body["data"]["attributes"]["token"]
            .as_str()
            .ok_or_else(|| Error::KeygenApiError {
                code: "INVALID_RESPONSE".to_string(),
                detail: "Invalid token response format".to_string(),
                body: response.body.clone(),
            })?;

        Ok(token_data.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData};

    #[test]
    fn test_product_account_relationship() {
        // Test that account_id is properly extracted from relationships
        let product_data = KeygenResponseData {
            id: "test-product-id".to_string(),
            r#type: "products".to_string(),
            attributes: ProductAttributes {
                name: "Test Product".to_string(),
                code: Some("test-product".to_string()),
                distribution_strategy: Some(DistributionStrategy::Open),
                url: Some("https://example.com".to_string()),
                platforms: Some(vec![Platform::Windows, Platform::MacOs]),
                permissions: Some(vec![Permission::LicenseRead, Permission::LicenseCreate]),
                metadata: Some(HashMap::new()),
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
                ..Default::default()
            },
        };

        let product = Product::from(product_data);
        
        assert_eq!(product.account_id, Some("test-account-id".to_string()));
        assert_eq!(product.id, "test-product-id");
        assert_eq!(product.name, "Test Product");
    }

    #[test]
    fn test_product_without_account_relationship() {
        // Test that account_id is None when no account relationship exists
        let product_data = KeygenResponseData {
            id: "test-product-id".to_string(),
            r#type: "products".to_string(),
            attributes: ProductAttributes {
                name: "Test Product".to_string(),
                code: Some("test-product".to_string()),
                distribution_strategy: Some(DistributionStrategy::Open),
                url: None,
                platforms: None,
                permissions: None,
                metadata: None,
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships::default(),
        };

        let product = Product::from(product_data);
        
        assert_eq!(product.account_id, None);
    }
}
