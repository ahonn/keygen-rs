#[cfg(feature = "token")]
use crate::client::Client;
#[cfg(feature = "token")]
use crate::errors::Error;
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentAttributes {
    pub fingerprint: String,
    pub name: String,
    pub metadata: Option<HashMap<String, Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ComponentResponse {
    pub data: KeygenResponseData<ComponentAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ComponentsResponse {
    pub data: Vec<KeygenResponseData<ComponentAttributes>>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateComponentRequest {
    pub fingerprint: String,
    pub name: String,
    pub metadata: Option<HashMap<String, Value>>,
    pub machine_id: String, // Required according to API docs
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListComponentsOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
    // Filters as per API documentation
    pub machine: Option<String>,
    pub license: Option<String>,
    pub owner: Option<String>,
    pub user: Option<String>,
    pub product: Option<String>,
}

#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateComponentRequest {
    pub name: Option<String>, // Only name and metadata are updatable
    pub metadata: Option<HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub fingerprint: String,
    pub name: String,
    pub metadata: Option<HashMap<String, Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    // Relationships as per API documentation
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub machine_id: Option<String>,
}

impl Default for Component {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: String::new(),
            fingerprint: String::new(),
            name: String::new(),
            metadata: None,
            created: now,
            updated: now,
            account_id: None,
            environment_id: None,
            product_id: None,
            license_id: None,
            machine_id: None,
        }
    }
}

impl Component {
    #[allow(dead_code)]
    pub(crate) fn from(data: KeygenResponseData<ComponentAttributes>) -> Component {
        Component {
            id: data.id,
            fingerprint: data.attributes.fingerprint,
            name: data.attributes.name,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
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
            product_id: data
                .relationships
                .product
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            license_id: data
                .relationships
                .license
                .as_ref()
                .and_then(|l| l.data.as_ref().map(|d| d.id.clone())),
            machine_id: data
                .relationships
                .machines
                .as_ref()
                .and_then(|m| m.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a simple component object for legacy compatibility
    pub fn create_object(component: &Component) -> serde_json::Value {
        json!({
          "data": {
            "id": component.id,
            "type": "components",
            "attributes": {
                "fingerprint": component.fingerprint,
                "name": component.name,
                "metadata": component.metadata
            }
          }
        })
    }

    /// Create a new component
    #[cfg(feature = "token")]
    pub async fn create(request: CreateComponentRequest) -> Result<Component, Error> {
        let client = Client::default()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert(
            "fingerprint".to_string(),
            Value::String(request.fingerprint),
        );
        attributes.insert("name".to_string(), Value::String(request.name));

        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = json!({
            "data": {
                "type": "components",
                "attributes": attributes,
                "relationships": {
                    "machine": {
                        "data": {
                            "type": "machines",
                            "id": request.machine_id
                        }
                    }
                }
            }
        });

        let response = client.post("components", Some(&body), None::<&()>).await?;
        let component_response: ComponentResponse = serde_json::from_value(response.body)?;
        Ok(Component::from(component_response.data))
    }

    /// List all components with optional filtering
    #[cfg(feature = "token")]
    pub async fn list(options: Option<ListComponentsOptions>) -> Result<Vec<Component>, Error> {
        let client = Client::default()?;
        let mut query_params = HashMap::new();

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query_params.insert("limit".to_string(), limit.to_string());
            }
            if let Some(page_size) = opts.page_size {
                query_params.insert("page[size]".to_string(), page_size.to_string());
            }
            if let Some(page_number) = opts.page_number {
                query_params.insert("page[number]".to_string(), page_number.to_string());
            }
            // API documented filters
            if let Some(machine) = opts.machine {
                query_params.insert("machine".to_string(), machine);
            }
            if let Some(license) = opts.license {
                query_params.insert("license".to_string(), license);
            }
            if let Some(owner) = opts.owner {
                query_params.insert("owner".to_string(), owner);
            }
            if let Some(user) = opts.user {
                query_params.insert("user".to_string(), user);
            }
            if let Some(product) = opts.product {
                query_params.insert("product".to_string(), product);
            }
        }

        let query = if query_params.is_empty() {
            None
        } else {
            Some(query_params)
        };

        let response = client.get("components", query.as_ref()).await?;
        let components_response: ComponentsResponse = serde_json::from_value(response.body)?;
        Ok(components_response
            .data
            .into_iter()
            .map(Component::from)
            .collect())
    }

    /// Get a component by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<Component, Error> {
        let client = Client::default()?;
        let endpoint = format!("components/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let component_response: ComponentResponse = serde_json::from_value(response.body)?;
        Ok(Component::from(component_response.data))
    }

    /// Update a component (only name and metadata are updatable per API docs)
    #[cfg(feature = "token")]
    pub async fn update(&self, request: UpdateComponentRequest) -> Result<Component, Error> {
        let client = Client::default()?;
        let endpoint = format!("components/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), Value::String(name));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = json!({
            "data": {
                "type": "components",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let component_response: ComponentResponse = serde_json::from_value(response.body)?;
        Ok(Component::from(component_response.data))
    }

    /// Delete a component
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("components/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }
}

// Convenience implementations for request builders
#[cfg(feature = "token")]
impl CreateComponentRequest {
    /// Create a new component creation request (machine_id is required)
    pub fn new(fingerprint: String, name: String, machine_id: String) -> Self {
        Self {
            fingerprint,
            name,
            metadata: None,
            machine_id,
        }
    }

    /// Set metadata for the component
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(feature = "token")]
impl UpdateComponentRequest {
    /// Create a new empty component update request
    pub fn new() -> Self {
        Self {
            name: None,
            metadata: None,
        }
    }

    /// Set the component name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the component metadata
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[cfg(feature = "token")]
impl ListComponentsOptions {
    /// Create new list options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit for number of results (1-100)
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set pagination options
    pub fn with_pagination(mut self, page_number: u32, page_size: u32) -> Self {
        self.page_number = Some(page_number);
        self.page_size = Some(page_size);
        self
    }

    /// Filter by machine ID
    pub fn with_machine(mut self, machine: String) -> Self {
        self.machine = Some(machine);
        self
    }

    /// Filter by license ID
    pub fn with_license(mut self, license: String) -> Self {
        self.license = Some(license);
        self
    }

    /// Filter by owner ID
    pub fn with_owner(mut self, owner: String) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Filter by user ID
    pub fn with_user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    /// Filter by product ID
    pub fn with_product(mut self, product: String) -> Self {
        self.product = Some(product);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };
    use chrono::Utc;

    #[test]
    fn test_component_from_data() {
        let component_data = KeygenResponseData {
            id: "test-component-id".to_string(),
            r#type: "components".to_string(),
            attributes: ComponentAttributes {
                fingerprint: "test-fingerprint".to_string(),
                name: "Test Component".to_string(),
                metadata: Some({
                    let mut map = HashMap::new();
                    map.insert("version".to_string(), Value::String("1.0.0".to_string()));
                    map
                }),
                created: Utc::now(),
                updated: Utc::now(),
            },
            relationships: KeygenRelationships {
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                environment: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "environments".to_string(),
                        id: "test-environment-id".to_string(),
                    }),
                    links: None,
                }),
                product: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    }),
                    links: None,
                }),
                license: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "licenses".to_string(),
                        id: "test-license-id".to_string(),
                    }),
                    links: None,
                }),
                machines: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "machines".to_string(),
                        id: "test-machine-id".to_string(),
                    }),
                    links: None,
                }),
                ..Default::default()
            },
        };

        let component = Component::from(component_data);

        assert_eq!(component.id, "test-component-id");
        assert_eq!(component.fingerprint, "test-fingerprint");
        assert_eq!(component.name, "Test Component");
        assert_eq!(component.account_id, Some("test-account-id".to_string()));
        assert_eq!(
            component.environment_id,
            Some("test-environment-id".to_string())
        );
        assert_eq!(component.product_id, Some("test-product-id".to_string()));
        assert_eq!(component.license_id, Some("test-license-id".to_string()));
        assert_eq!(component.machine_id, Some("test-machine-id".to_string()));
        assert!(component.metadata.is_some());
        let metadata = component.metadata.unwrap();
        assert_eq!(metadata.get("version").unwrap().as_str().unwrap(), "1.0.0");
    }

    #[test]
    fn test_component_without_relationships() {
        let component_data = KeygenResponseData {
            id: "test-component-id".to_string(),
            r#type: "components".to_string(),
            attributes: ComponentAttributes {
                fingerprint: "test-fingerprint".to_string(),
                name: "Test Component".to_string(),
                metadata: None,
                created: Utc::now(),
                updated: Utc::now(),
            },
            relationships: KeygenRelationships::default(),
        };

        let component = Component::from(component_data);

        assert_eq!(component.account_id, None);
        assert_eq!(component.environment_id, None);
        assert_eq!(component.product_id, None);
        assert_eq!(component.license_id, None);
        assert_eq!(component.machine_id, None);
        assert!(component.metadata.is_none());
    }

    #[cfg(feature = "token")]
    #[test]
    fn test_create_component_request_builder() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), Value::String("1.0.0".to_string()));

        let request = CreateComponentRequest::new(
            "test-fingerprint".to_string(),
            "Test Component".to_string(),
            "test-machine-id".to_string(),
        )
        .with_metadata(metadata.clone());

        assert_eq!(request.fingerprint, "test-fingerprint");
        assert_eq!(request.name, "Test Component");
        assert_eq!(request.machine_id, "test-machine-id");
        assert_eq!(request.metadata, Some(metadata));
    }

    #[cfg(feature = "token")]
    #[test]
    fn test_update_component_request_builder() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), Value::String("2.0.0".to_string()));

        let request = UpdateComponentRequest::new()
            .with_name("Updated Component".to_string())
            .with_metadata(metadata.clone());

        assert_eq!(request.name, Some("Updated Component".to_string()));
        assert_eq!(request.metadata, Some(metadata));
    }

    #[cfg(feature = "token")]
    #[test]
    fn test_list_components_options_builder() {
        let options = ListComponentsOptions::new()
            .with_limit(50)
            .with_pagination(2, 25)
            .with_machine("test-machine-id".to_string())
            .with_license("test-license-id".to_string())
            .with_owner("test-owner-id".to_string())
            .with_user("test-user-id".to_string())
            .with_product("test-product-id".to_string());

        assert_eq!(options.limit, Some(50));
        assert_eq!(options.page_number, Some(2));
        assert_eq!(options.page_size, Some(25));
        assert_eq!(options.machine, Some("test-machine-id".to_string()));
        assert_eq!(options.license, Some("test-license-id".to_string()));
        assert_eq!(options.owner, Some("test-owner-id".to_string()));
        assert_eq!(options.user, Some("test-user-id".to_string()));
        assert_eq!(options.product, Some("test-product-id".to_string()));
    }

    #[test]
    fn test_component_default() {
        let component = Component::default();

        assert!(component.id.is_empty());
        assert!(component.fingerprint.is_empty());
        assert!(component.name.is_empty());
        assert!(component.metadata.is_none());
        assert!(component.account_id.is_none());
        assert!(component.environment_id.is_none());
        assert!(component.product_id.is_none());
        assert!(component.license_id.is_none());
        assert!(component.machine_id.is_none());
        // created and updated should be set to now
        assert!(component.created <= Utc::now());
        assert!(component.updated <= Utc::now());
    }

    #[test]
    fn test_create_object_legacy_compatibility() {
        let component = Component {
            id: "test-id".to_string(),
            fingerprint: "test-fingerprint".to_string(),
            name: "Test Component".to_string(),
            metadata: Some({
                let mut map = HashMap::new();
                map.insert("version".to_string(), Value::String("1.0.0".to_string()));
                map
            }),
            ..Default::default()
        };

        let object = Component::create_object(&component);

        assert!(object["data"]["id"].is_string());
        assert_eq!(object["data"]["type"], "components");
        assert_eq!(
            object["data"]["attributes"]["fingerprint"],
            "test-fingerprint"
        );
        assert_eq!(object["data"]["attributes"]["name"], "Test Component");
    }
}
