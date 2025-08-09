use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IsolationStrategy {
    /// Complete resource isolation (default)
    Isolated,
    /// Global environment resources readable
    Shared,
}

impl Default for IsolationStrategy {
    fn default() -> Self {
        Self::Isolated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentAttributes {
    pub name: String,
    pub code: String,
    #[serde(rename = "isolationStrategy")]
    pub isolation_strategy: IsolationStrategy,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub code: String,
    pub isolation_strategy: IsolationStrategy,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnvironmentResponse {
    pub data: KeygenResponseData<EnvironmentAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnvironmentsResponse {
    pub data: Vec<KeygenResponseData<EnvironmentAttributes>>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentsListResult {
    pub environments: Vec<Environment>,
    pub meta: Option<serde_json::Value>,
    pub links: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub code: String,
    #[serde(rename = "isolationStrategy")]
    pub isolation_strategy: Option<IsolationStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEnvironmentRequest {
    pub name: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListEnvironmentsOptions {
    pub limit: Option<u32>,
    #[serde(rename = "page[size]")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]")]
    pub page_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEnvironmentTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentTokenAttributes {
    pub token: String,
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentToken {
    pub id: String,
    pub token: String,
    pub name: Option<String>,
    pub expiry: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub created: String,
    pub updated: String,
    pub environment_id: String,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EnvironmentTokenResponse {
    pub data: KeygenResponseData<EnvironmentTokenAttributes>,
}

impl Environment {
    pub(crate) fn from(data: KeygenResponseData<EnvironmentAttributes>) -> Environment {
        Environment {
            id: data.id,
            name: data.attributes.name,
            code: data.attributes.code,
            isolation_strategy: data.attributes.isolation_strategy,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// Create a new environment
    #[cfg(feature = "token")]
    pub async fn create(request: CreateEnvironmentRequest) -> Result<Environment, Error> {
        let client = Client::default()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert("name".to_string(), json!(request.name));
        attributes.insert("code".to_string(), json!(request.code));

        if let Some(isolation_strategy) = request.isolation_strategy {
            attributes.insert("isolationStrategy".to_string(), json!(isolation_strategy));
        }

        let body = json!({
            "data": {
                "type": "environments",
                "attributes": attributes
            }
        });

        let response = client
            .post("environments", Some(&body), None::<&()>)
            .await?;
        let environment_response: EnvironmentResponse = serde_json::from_value(response.body)?;
        Ok(Environment::from(environment_response.data))
    }

    /// List environments with optional filters
    #[cfg(feature = "token")]
    pub async fn list(
        options: Option<ListEnvironmentsOptions>,
    ) -> Result<EnvironmentsListResult, Error> {
        let client = Client::default()?;

        let mut query_params = HashMap::new();
        if let Some(options) = options {
            if let Some(limit) = options.limit {
                query_params.insert("limit".to_string(), limit.to_string());
            }
            if let Some(page_size) = options.page_size {
                query_params.insert("page[size]".to_string(), page_size.to_string());
            }
            if let Some(page_number) = options.page_number {
                query_params.insert("page[number]".to_string(), page_number.to_string());
            }
        }

        let query = if query_params.is_empty() {
            None
        } else {
            Some(query_params)
        };

        let response = client.get("environments", query.as_ref()).await?;
        let environments_response: EnvironmentsResponse = serde_json::from_value(response.body)?;

        Ok(EnvironmentsListResult {
            environments: environments_response
                .data
                .into_iter()
                .map(Environment::from)
                .collect(),
            meta: environments_response.meta,
            links: environments_response.links,
        })
    }

    /// Get an environment by ID or code
    #[cfg(feature = "token")]
    pub async fn get(id_or_code: &str) -> Result<Environment, Error> {
        let client = Client::default()?;
        let endpoint = format!("environments/{id_or_code}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let environment_response: EnvironmentResponse = serde_json::from_value(response.body)?;
        Ok(Environment::from(environment_response.data))
    }

    /// Update an environment
    #[cfg(feature = "token")]
    pub async fn update(&self, request: UpdateEnvironmentRequest) -> Result<Environment, Error> {
        let client = Client::default()?;
        let endpoint = format!("environments/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), json!(name));
        }
        if let Some(code) = request.code {
            attributes.insert("code".to_string(), json!(code));
        }

        let body = json!({
            "data": {
                "type": "environments",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let environment_response: EnvironmentResponse = serde_json::from_value(response.body)?;
        Ok(Environment::from(environment_response.data))
    }

    /// Delete an environment
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("environments/{}", self.id);
        let _response = client
            .delete::<(), serde_json::Value>(&endpoint, None::<&()>)
            .await?;
        Ok(())
    }

    /// Generate a token for this environment
    #[cfg(feature = "token")]
    pub async fn generate_token(
        &self,
        request: Option<CreateEnvironmentTokenRequest>,
    ) -> Result<EnvironmentToken, Error> {
        let client = Client::default()?;
        let endpoint = format!("environments/{}/tokens", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(request) = request {
            if let Some(name) = request.name {
                attributes.insert("name".to_string(), json!(name));
            }
            if let Some(expiry) = request.expiry {
                attributes.insert("expiry".to_string(), json!(expiry));
            }
            if let Some(permissions) = request.permissions {
                attributes.insert("permissions".to_string(), json!(permissions));
            }
        }

        let body = json!({
            "data": {
                "type": "tokens",
                "attributes": attributes
            }
        });

        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let token_response: EnvironmentTokenResponse = serde_json::from_value(response.body)?;

        Ok(EnvironmentToken {
            id: token_response.data.id,
            token: token_response.data.attributes.token,
            name: token_response.data.attributes.name,
            expiry: token_response.data.attributes.expiry,
            permissions: token_response.data.attributes.permissions,
            created: token_response.data.attributes.created,
            updated: token_response.data.attributes.updated,
            environment_id: self.id.clone(),
            account_id: self.account_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_environment_from_response_data() {
        let environment_data = KeygenResponseData {
            id: "test-environment-id".to_string(),
            r#type: "environments".to_string(),
            attributes: EnvironmentAttributes {
                name: "Test Environment".to_string(),
                code: "test-env".to_string(),
                isolation_strategy: IsolationStrategy::Isolated,
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
            },
            relationships: KeygenRelationships {
                policy: None,
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                product: None,
                group: None,
                owner: None,
                users: None,
                machines: None,
                environment: None,
                license: None,
                other: HashMap::new(),
            },
        };

        let environment = Environment::from(environment_data);

        assert_eq!(environment.id, "test-environment-id");
        assert_eq!(environment.name, "Test Environment");
        assert_eq!(environment.code, "test-env");
        assert_eq!(environment.isolation_strategy, IsolationStrategy::Isolated);
        assert_eq!(environment.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_isolation_strategy_default() {
        let default_strategy = IsolationStrategy::default();
        assert_eq!(default_strategy, IsolationStrategy::Isolated);
    }

    #[test]
    fn test_create_environment_request_serialization() {
        let request = CreateEnvironmentRequest {
            name: "Production".to_string(),
            code: "prod".to_string(),
            isolation_strategy: Some(IsolationStrategy::Shared),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Production\""));
        assert!(json.contains("\"code\":\"prod\""));
        assert!(json.contains("\"isolationStrategy\":\"SHARED\""));
    }

    #[test]
    fn test_create_environment_token_request_optional_fields() {
        let request = CreateEnvironmentTokenRequest {
            name: Some("Test Token".to_string()),
            expiry: None,
            permissions: Some(vec!["environment.read".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Test Token\""));
        assert!(json.contains("\"permissions\":[\"environment.read\"]"));
        assert!(!json.contains("expiry"));
    }
}
