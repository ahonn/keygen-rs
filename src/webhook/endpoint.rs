use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;

use super::event::{WebhookEventRecord, WebhookEventResponse, WebhookEventsResponse};
use super::event_types::WebhookEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    #[serde(rename = "ED25519", alias = "ed25519")]
    Ed25519,
    #[serde(rename = "RSA_2048_PSS_SHA256", alias = "rsa_2048_pss_sha256")]
    Rsa2048PssSha256,
    #[serde(rename = "RSA_2048_PKCS1_SHA256", alias = "rsa_2048_pkcs1_sha256")]
    Rsa2048Pkcs1Sha256,
}

impl Default for SignatureAlgorithm {
    fn default() -> Self {
        Self::Ed25519
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WebhookEndpointAttributes {
    pub url: String,
    #[serde(default)]
    pub subscriptions: Vec<String>,
    #[serde(rename = "signatureAlgorithm")]
    pub signature_algorithm: SignatureAlgorithm,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WebhookEndpointResponse {
    pub data: KeygenResponseData<WebhookEndpointAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEndpoint {
    pub id: String,
    pub url: String,
    pub subscriptions: Vec<String>,
    pub signature_algorithm: SignatureAlgorithm,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
}

/// Options for listing webhook endpoints
#[derive(Debug, Default)]
pub struct WebhookEndpointListOptions {
    pub limit: Option<i32>,
    pub page_number: Option<i32>,
    pub page_size: Option<i32>,
}

/// Request for creating a webhook endpoint
#[derive(Debug)]
pub struct WebhookEndpointCreateRequest {
    pub url: String,
    pub subscriptions: Option<Vec<WebhookEvent>>,
    pub signature_algorithm: Option<SignatureAlgorithm>,
    pub environment_id: Option<String>,
}

/// Request for updating a webhook endpoint
#[derive(Debug, Default)]
pub struct WebhookEndpointUpdateRequest {
    pub url: Option<String>,
    pub subscriptions: Option<Vec<WebhookEvent>>,
    pub signature_algorithm: Option<SignatureAlgorithm>,
}

impl WebhookEndpointCreateRequest {
    /// Create a new webhook endpoint creation request with the required URL
    pub fn new(url: String) -> Self {
        Self {
            url,
            subscriptions: None,
            signature_algorithm: None,
            environment_id: None,
        }
    }

    /// Set the event subscriptions
    pub fn with_subscriptions(mut self, subscriptions: Vec<WebhookEvent>) -> Self {
        self.subscriptions = Some(subscriptions);
        self
    }

    /// Set the signature algorithm
    pub fn with_signature_algorithm(mut self, algorithm: SignatureAlgorithm) -> Self {
        self.signature_algorithm = Some(algorithm);
        self
    }

    /// Set the environment ID
    pub fn with_environment_id(mut self, environment_id: String) -> Self {
        self.environment_id = Some(environment_id);
        self
    }

    /// Convert to JSON body for API request
    pub fn to_json_body(self) -> Value {
        let mut attributes = serde_json::Map::new();
        let mut relationships = serde_json::Map::new();

        attributes.insert("url".to_string(), json!(self.url));

        if let Some(subscriptions) = self.subscriptions {
            let subscription_strings: Vec<String> = subscriptions
                .into_iter()
                .map(|event| {
                    // Convert enum to string representation
                    serde_json::to_value(&event)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "*".to_string())
                })
                .collect();
            attributes.insert("subscriptions".to_string(), json!(subscription_strings));
        }

        if let Some(algorithm) = self.signature_algorithm {
            attributes.insert("signatureAlgorithm".to_string(), json!(algorithm));
        }

        if let Some(environment_id) = self.environment_id {
            relationships.insert(
                "environment".to_string(),
                json!({
                    "data": {
                        "type": "environments",
                        "id": environment_id
                    }
                }),
            );
        }

        let mut data = json!({
            "type": "webhook-endpoints",
            "attributes": attributes
        });

        if !relationships.is_empty() {
            data["relationships"] = json!(relationships);
        }

        json!({ "data": data })
    }
}

impl WebhookEndpointUpdateRequest {
    /// Create a new webhook endpoint update request
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the URL
    pub fn with_url(mut self, url: String) -> Self {
        self.url = Some(url);
        self
    }

    /// Set the event subscriptions
    pub fn with_subscriptions(mut self, subscriptions: Vec<WebhookEvent>) -> Self {
        self.subscriptions = Some(subscriptions);
        self
    }

    /// Set the signature algorithm
    pub fn with_signature_algorithm(mut self, algorithm: SignatureAlgorithm) -> Self {
        self.signature_algorithm = Some(algorithm);
        self
    }

    /// Convert to JSON body for API request
    pub fn to_json_body(self) -> Value {
        let mut attributes = serde_json::Map::new();

        if let Some(url) = self.url {
            attributes.insert("url".to_string(), json!(url));
        }

        if let Some(subscriptions) = self.subscriptions {
            let subscription_strings: Vec<String> = subscriptions
                .into_iter()
                .map(|event| {
                    serde_json::to_value(&event)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_else(|| "*".to_string())
                })
                .collect();
            attributes.insert("subscriptions".to_string(), json!(subscription_strings));
        }

        if let Some(algorithm) = self.signature_algorithm {
            attributes.insert("signatureAlgorithm".to_string(), json!(algorithm));
        }

        json!({
            "data": {
                "type": "webhook-endpoints",
                "attributes": attributes
            }
        })
    }
}

impl WebhookEndpoint {
    pub(crate) fn from(data: KeygenResponseData<WebhookEndpointAttributes>) -> Self {
        WebhookEndpoint {
            id: data.id,
            url: data.attributes.url,
            subscriptions: data.attributes.subscriptions,
            signature_algorithm: data.attributes.signature_algorithm,
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
        }
    }

    /// Create a new webhook endpoint
    #[cfg(feature = "token")]
    pub async fn create(request: WebhookEndpointCreateRequest) -> Result<WebhookEndpoint, Error> {
        let client = Client::default()?;
        let body = request.to_json_body();
        let response = client
            .post("webhook-endpoints", Some(&body), None::<&()>)
            .await?;
        let endpoint_response: WebhookEndpointResponse = serde_json::from_value(response.body)?;
        Ok(WebhookEndpoint::from(endpoint_response.data))
    }

    /// List all webhook endpoints
    #[cfg(feature = "token")]
    pub async fn list(
        options: Option<&WebhookEndpointListOptions>,
    ) -> Result<Vec<WebhookEndpoint>, Error> {
        let client = Client::default()?;
        let mut query = json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            }
            if let Some(page_number) = opts.page_number {
                query["page[number]"] = json!(page_number);
            }
            if let Some(page_size) = opts.page_size {
                query["page[size]"] = json!(page_size);
            }
        }

        let response = client.get("webhook-endpoints", Some(&query)).await?;

        #[derive(Debug, Clone, Serialize, Deserialize)]
        struct WebhookEndpointsResponse {
            pub data: Vec<KeygenResponseData<WebhookEndpointAttributes>>,
        }

        let endpoints_response: WebhookEndpointsResponse = serde_json::from_value(response.body)?;
        Ok(endpoints_response
            .data
            .into_iter()
            .map(WebhookEndpoint::from)
            .collect())
    }

    /// Get a webhook endpoint by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<WebhookEndpoint, Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-endpoints/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let endpoint_response: WebhookEndpointResponse = serde_json::from_value(response.body)?;
        Ok(WebhookEndpoint::from(endpoint_response.data))
    }

    /// Update a webhook endpoint
    #[cfg(feature = "token")]
    pub async fn update(
        &self,
        request: WebhookEndpointUpdateRequest,
    ) -> Result<WebhookEndpoint, Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-endpoints/{}", self.id);
        let body = request.to_json_body();
        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let endpoint_response: WebhookEndpointResponse = serde_json::from_value(response.body)?;
        Ok(WebhookEndpoint::from(endpoint_response.data))
    }

    /// Delete a webhook endpoint
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-endpoints/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// List webhook events for this endpoint
    #[cfg(feature = "token")]
    pub async fn events(
        &self,
        options: Option<&WebhookEndpointListOptions>,
    ) -> Result<Vec<WebhookEventRecord>, Error> {
        let client = Client::default()?;
        let mut query = json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = json!(limit);
            }
            if let Some(page_number) = opts.page_number {
                query["page[number]"] = json!(page_number);
            }
            if let Some(page_size) = opts.page_size {
                query["page[size]"] = json!(page_size);
            }
        }

        let endpoint = format!("webhook-endpoints/{}/webhook-events", self.id);
        let response = client.get(&endpoint, Some(&query)).await?;

        let events_response: WebhookEventsResponse = serde_json::from_value(response.body)?;
        Ok(events_response
            .data
            .into_iter()
            .map(WebhookEventRecord::from)
            .collect())
    }

    /// Retry a specific webhook event
    #[cfg(feature = "token")]
    pub async fn retry_event(&self, event_id: &str) -> Result<WebhookEventRecord, Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-events/{}/actions/retry", event_id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;

        let event_response: WebhookEventResponse = serde_json::from_value(response.body)?;
        Ok(WebhookEventRecord::from(event_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{reset_config, set_config, KeygenConfig};
    use mockito::{mock, server_url};

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_create_webhook_endpoint() {
        let _m = mock("POST", "/v1/webhook-endpoints")
            .match_header("authorization", "Bearer admin-token")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "webhook-123",
                        "type": "webhook-endpoints",
                        "attributes": {
                            "url": "https://example.com/webhook",
                            "subscriptions": ["license.created", "license.updated"],
                            "signatureAlgorithm": "ED25519",
                            "created": "2024-01-01T00:00:00Z",
                            "updated": "2024-01-01T00:00:00Z"
                        },
                        "relationships": {}
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let request = WebhookEndpointCreateRequest::new("https://example.com/webhook".to_string())
            .with_subscriptions(vec![
                WebhookEvent::LicenseCreated,
                WebhookEvent::LicenseUpdated,
            ])
            .with_signature_algorithm(SignatureAlgorithm::Ed25519);

        let result = WebhookEndpoint::create(request).await;
        assert!(result.is_ok());
        let endpoint = result.unwrap();
        assert_eq!(endpoint.id, "webhook-123");
        assert_eq!(endpoint.url, "https://example.com/webhook");

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_list_webhook_endpoints() {
        let _m = mock("GET", "/v1/webhook-endpoints")
            .match_header("authorization", "Bearer admin-token")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "webhook-1",
                            "type": "webhook-endpoints",
                            "attributes": {
                                "url": "https://example.com/webhook1",
                                "subscriptions": ["*"],
                                "signatureAlgorithm": "ED25519",
                                "created": "2024-01-01T00:00:00Z",
                                "updated": "2024-01-01T00:00:00Z"
                            },
                            "relationships": {}
                        },
                        {
                            "id": "webhook-2",
                            "type": "webhook-endpoints",
                            "attributes": {
                                "url": "https://example.com/webhook2",
                                "subscriptions": ["license.created"],
                                "signatureAlgorithm": "RSA_2048_PSS_SHA256",
                                "created": "2024-01-01T00:00:00Z",
                                "updated": "2024-01-01T00:00:00Z"
                            },
                            "relationships": {}
                        }
                    ]
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let result = WebhookEndpoint::list(None).await;
        assert!(result.is_ok());
        let endpoints = result.unwrap();
        assert_eq!(endpoints.len(), 2);
        assert_eq!(endpoints[0].id, "webhook-1");
        assert_eq!(endpoints[1].id, "webhook-2");

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_update_webhook_endpoint() {
        let endpoint = WebhookEndpoint {
            id: "webhook-123".to_string(),
            url: "https://example.com/webhook".to_string(),
            subscriptions: vec!["license.created".to_string()],
            signature_algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now(),
            updated: Utc::now(),
            account_id: None,
            environment_id: None,
        };

        let _m = mock("PATCH", "/v1/webhook-endpoints/webhook-123")
            .match_header("authorization", "Bearer admin-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "webhook-123",
                        "type": "webhook-endpoints",
                        "attributes": {
                            "url": "https://example.com/new-webhook",
                            "subscriptions": ["license.created", "license.deleted"],
                            "signatureAlgorithm": "ED25519",
                            "created": "2024-01-01T00:00:00Z",
                            "updated": "2024-01-02T00:00:00Z"
                        },
                        "relationships": {}
                    }
                })
                .to_string(),
            )
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let request = WebhookEndpointUpdateRequest::new()
            .with_url("https://example.com/new-webhook".to_string())
            .with_subscriptions(vec![
                WebhookEvent::LicenseCreated,
                WebhookEvent::LicenseDeleted,
            ]);

        let result = endpoint.update(request).await;
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.url, "https://example.com/new-webhook");
        assert_eq!(updated.subscriptions.len(), 2);

        reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_delete_webhook_endpoint() {
        let endpoint = WebhookEndpoint {
            id: "webhook-123".to_string(),
            url: "https://example.com/webhook".to_string(),
            subscriptions: vec!["*".to_string()],
            signature_algorithm: SignatureAlgorithm::Ed25519,
            created: Utc::now(),
            updated: Utc::now(),
            account_id: None,
            environment_id: None,
        };

        let _m = mock("DELETE", "/v1/webhook-endpoints/webhook-123")
            .match_header("authorization", "Bearer admin-token")
            .with_status(204)
            .create();

        let _ = set_config(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            token: Some("admin-token".to_string()),
            ..Default::default()
        });

        let result = endpoint.delete().await;
        assert!(result.is_ok());

        reset_config();
    }

    #[test]
    fn test_webhook_event_serialization() {
        let event = WebhookEvent::LicenseCreated;
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json, "license.created");

        let event = WebhookEvent::MachineHeartbeatPing;
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json, "machine.heartbeat.ping");

        let event = WebhookEvent::All;
        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json, "*");
    }

    #[test]
    fn test_signature_algorithm_serialization() {
        let algo = SignatureAlgorithm::Ed25519;
        let json = serde_json::to_value(&algo).unwrap();
        assert_eq!(json, "ED25519");

        let algo = SignatureAlgorithm::Rsa2048PssSha256;
        let json = serde_json::to_value(&algo).unwrap();
        assert_eq!(json, "RSA_2048_PSS_SHA256");
    }
}
