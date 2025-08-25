use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebhookEventStatus {
    #[serde(rename = "DELIVERING")]
    Delivering,
    #[serde(rename = "DELIVERED")]
    Delivered,
    #[serde(rename = "FAILING")]
    Failing,
    #[serde(rename = "FAILED")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WebhookEventAttributes {
    pub endpoint: String,
    pub payload: Value,
    pub event: String,
    pub status: WebhookEventStatus,
    #[serde(rename = "lastResponseCode")]
    pub last_response_code: Option<i32>,
    #[serde(rename = "lastResponseBody")]
    pub last_response_body: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WebhookEventResponse {
    pub data: KeygenResponseData<WebhookEventAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct WebhookEventsResponse {
    pub data: Vec<KeygenResponseData<WebhookEventAttributes>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEventRecord {
    pub id: String,
    pub endpoint: String,
    pub payload: Value,
    pub event: String,
    pub status: WebhookEventStatus,
    pub last_response_code: Option<i32>,
    pub last_response_body: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
}

#[derive(Debug, Default)]
pub struct WebhookEventListOptions {
    pub limit: Option<i32>,
    pub page_number: Option<i32>,
    pub page_size: Option<i32>,
    pub event_type: Option<String>,
    pub status: Option<WebhookEventStatus>,
}

impl WebhookEventRecord {
    pub(crate) fn from(data: KeygenResponseData<WebhookEventAttributes>) -> Self {
        WebhookEventRecord {
            id: data.id,
            endpoint: data.attributes.endpoint,
            payload: data.attributes.payload,
            event: data.attributes.event,
            status: data.attributes.status,
            last_response_code: data.attributes.last_response_code,
            last_response_body: data.attributes.last_response_body,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// List all webhook events
    #[cfg(feature = "token")]
    pub async fn list(
        options: Option<&WebhookEventListOptions>,
    ) -> Result<Vec<WebhookEventRecord>, Error> {
        let client = Client::default()?;
        let mut query = serde_json::json!({});

        if let Some(opts) = options {
            if let Some(limit) = opts.limit {
                query["limit"] = serde_json::json!(limit);
            }
            if let Some(page_number) = opts.page_number {
                query["page[number]"] = serde_json::json!(page_number);
            }
            if let Some(page_size) = opts.page_size {
                query["page[size]"] = serde_json::json!(page_size);
            }
            if let Some(ref event_type) = opts.event_type {
                query["event"] = serde_json::json!(event_type);
            }
            if let Some(ref status) = opts.status {
                query["status"] = serde_json::to_value(status).unwrap_or_default();
            }
        }

        let response = client.get("webhook-events", Some(&query)).await?;
        let events_response: WebhookEventsResponse = serde_json::from_value(response.body)?;
        Ok(events_response
            .data
            .into_iter()
            .map(WebhookEventRecord::from)
            .collect())
    }

    /// Get a webhook event by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<WebhookEventRecord, Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-events/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let event_response: WebhookEventResponse = serde_json::from_value(response.body)?;
        Ok(WebhookEventRecord::from(event_response.data))
    }

    /// Retry this webhook event
    #[cfg(feature = "token")]
    pub async fn retry(&self) -> Result<WebhookEventRecord, Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-events/{}/actions/retry", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let event_response: WebhookEventResponse = serde_json::from_value(response.body)?;
        Ok(WebhookEventRecord::from(event_response.data))
    }

    /// Delete this webhook event
    #[cfg(feature = "token")]
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::default()?;
        let endpoint = format!("webhook-events/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Check if the event is in a delivered state
    pub fn is_delivered(&self) -> bool {
        matches!(self.status, WebhookEventStatus::Delivered)
    }

    /// Check if the event failed to deliver
    pub fn is_failed(&self) -> bool {
        matches!(self.status, WebhookEventStatus::Failed)
    }

    /// Check if the event is currently being delivered or failing
    pub fn is_pending(&self) -> bool {
        matches!(
            self.status,
            WebhookEventStatus::Delivering | WebhookEventStatus::Failing
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{reset_config, set_config, KeygenConfig};
    use mockito::{mock, server_url};
    use serde_json::json;

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_list_webhook_events() {
        let _m = mock("GET", "/v1/webhook-events")
            .match_header("authorization", "Bearer admin-token")
            .match_query(mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": [
                        {
                            "id": "event-1",
                            "type": "webhook-events",
                            "meta": {
                                "idempotencyToken": "token-123"
                            },
                            "attributes": {
                                "endpoint": "https://example.com/webhook",
                                "payload": {"test": "data"},
                                "event": "license.created",
                                "status": "DELIVERED",
                                "lastResponseCode": 200,
                                "lastResponseBody": "OK",
                                "created": "2024-01-01T00:00:00Z",
                                "updated": "2024-01-01T00:00:01Z"
                            },
                            "relationships": {
                                "account": {
                                    "data": {
                                        "type": "accounts",
                                        "id": "account-123"
                                    }
                                }
                            }
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

        let result = WebhookEventRecord::list(None).await;
        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "event-1");
        assert_eq!(events[0].event, "license.created");
        assert!(events[0].is_delivered());

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_get_webhook_event() {
        let _m = mock("GET", "/v1/webhook-events/event-123")
            .match_header("authorization", "Bearer admin-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "event-123",
                        "type": "webhook-events",
                        "attributes": {
                            "endpoint": "https://example.com/webhook",
                            "payload": {"license": {"id": "license-456"}},
                            "event": "license.validation.succeeded",
                            "status": "FAILED",
                            "lastResponseCode": 500,
                            "lastResponseBody": "Internal Server Error",
                            "created": "2024-01-01T00:00:00Z",
                            "updated": "2024-01-01T00:00:10Z"
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

        let result = WebhookEventRecord::get("event-123").await;
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.id, "event-123");
        assert_eq!(event.event, "license.validation.succeeded");
        assert!(event.is_failed());
        assert_eq!(event.last_response_code, Some(500));

        let _ = reset_config();
    }

    #[cfg(feature = "token")]
    #[tokio::test]
    async fn test_retry_webhook_event() {
        let event = WebhookEventRecord {
            id: "event-123".to_string(),
            endpoint: "https://example.com/webhook".to_string(),
            payload: json!({"test": "data"}),
            event: "license.created".to_string(),
            status: WebhookEventStatus::Failed,
            last_response_code: Some(500),
            last_response_body: Some("Error".to_string()),
            created: Utc::now(),
            updated: Utc::now(),
            account_id: None,
        };

        let _m = mock("POST", "/v1/webhook-events/event-123/actions/retry")
            .match_header("authorization", "Bearer admin-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "data": {
                        "id": "event-123",
                        "type": "webhook-events",
                        "attributes": {
                            "endpoint": "https://example.com/webhook",
                            "payload": {"test": "data"},
                            "event": "license.created",
                            "status": "DELIVERING",
                            "lastResponseCode": null,
                            "lastResponseBody": null,
                            "created": "2024-01-01T00:00:00Z",
                            "updated": "2024-01-01T00:00:15Z"
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

        let result = event.retry().await;
        assert!(result.is_ok());
        let retried = result.unwrap();
        assert!(retried.is_pending());
        assert_eq!(retried.last_response_code, None);

        let _ = reset_config();
    }

    #[test]
    fn test_webhook_event_status_checks() {
        let delivered_event = WebhookEventRecord {
            id: "1".to_string(),
            endpoint: "https://example.com".to_string(),
            payload: json!({}),
            event: "test".to_string(),
            status: WebhookEventStatus::Delivered,
            last_response_code: Some(200),
            last_response_body: None,
            created: Utc::now(),
            updated: Utc::now(),
            account_id: None,
        };

        assert!(delivered_event.is_delivered());
        assert!(!delivered_event.is_failed());
        assert!(!delivered_event.is_pending());

        let failed_event = WebhookEventRecord {
            status: WebhookEventStatus::Failed,
            ..delivered_event.clone()
        };

        assert!(!failed_event.is_delivered());
        assert!(failed_event.is_failed());
        assert!(!failed_event.is_pending());

        let delivering_event = WebhookEventRecord {
            status: WebhookEventStatus::Delivering,
            ..delivered_event
        };

        assert!(!delivering_event.is_delivered());
        assert!(!delivering_event.is_failed());
        assert!(delivering_event.is_pending());
    }
}
