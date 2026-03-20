use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct WebhookEndpoint {
    pub id: String,
    pub url: String,
    pub subscriptions: Vec<String>,
    pub signature_algorithm: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
}

impl From<keygen_rs::webhook::endpoint::WebhookEndpoint> for WebhookEndpoint {
    fn from(e: keygen_rs::webhook::endpoint::WebhookEndpoint) -> Self {
        let signature_algorithm = serde_json::to_value(&e.signature_algorithm)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "ed25519".to_string());
        WebhookEndpoint {
            id: e.id,
            url: e.url,
            subscriptions: e.subscriptions,
            signature_algorithm,
            created: e.created.to_rfc3339(),
            updated: e.updated.to_rfc3339(),
            account_id: e.account_id,
            environment_id: e.environment_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct WebhookEvent {
    pub id: String,
    pub endpoint: String,
    pub payload: serde_json::Value,
    pub event: String,
    pub status: String,
    pub last_response_code: Option<i32>,
    pub last_response_body: Option<String>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::webhook::event::WebhookEventRecord> for WebhookEvent {
    fn from(e: keygen_rs::webhook::event::WebhookEventRecord) -> Self {
        let status = serde_json::to_value(&e.status)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        WebhookEvent {
            id: e.id,
            endpoint: e.endpoint,
            payload: e.payload,
            event: e.event,
            status,
            last_response_code: e.last_response_code,
            last_response_body: e.last_response_body,
            created: e.created.to_rfc3339(),
            updated: e.updated.to_rfc3339(),
            account_id: e.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateWebhookEndpointRequest {
    pub url: Option<String>,
    pub subscriptions: Option<Vec<String>>,
    pub signature_algorithm: Option<String>,
    pub environment_id: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateWebhookEndpointRequest {
    pub url: Option<String>,
    pub subscriptions: Option<Vec<String>>,
    pub signature_algorithm: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListWebhookEndpointsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListWebhookEventsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub event_type: Option<String>,
    pub status: Option<String>,
}

fn parse_signature_algorithm(
    s: &str,
) -> std::result::Result<keygen_rs::webhook::endpoint::SignatureAlgorithm, napi::Error> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).map_err(|e| {
        napi::Error::new(
            Status::InvalidArg,
            format!("Invalid signature algorithm: {e}"),
        )
    })
}

fn parse_webhook_events(subs: Vec<String>) -> Vec<keygen_rs::webhook::event_types::WebhookEvent> {
    subs.into_iter()
        .filter_map(|s| {
            serde_json::from_value::<keygen_rs::webhook::event_types::WebhookEvent>(
                serde_json::Value::String(s),
            )
            .ok()
        })
        .collect()
}

fn make_minimal_endpoint(id: String) -> keygen_rs::webhook::endpoint::WebhookEndpoint {
    keygen_rs::webhook::endpoint::WebhookEndpoint {
        id,
        url: String::new(),
        subscriptions: Vec::new(),
        signature_algorithm: keygen_rs::webhook::endpoint::SignatureAlgorithm::Ed25519,
        created: chrono::Utc::now(),
        updated: chrono::Utc::now(),
        account_id: None,
        environment_id: None,
    }
}

fn make_minimal_event_record(id: String) -> keygen_rs::webhook::event::WebhookEventRecord {
    keygen_rs::webhook::event::WebhookEventRecord {
        id,
        endpoint: String::new(),
        payload: serde_json::Value::Null,
        event: String::new(),
        status: keygen_rs::webhook::event::WebhookEventStatus::Delivering,
        last_response_code: None,
        last_response_body: None,
        created: chrono::Utc::now(),
        updated: chrono::Utc::now(),
        account_id: None,
    }
}

#[napi]
pub async fn create_webhook_endpoint(
    request: CreateWebhookEndpointRequest,
) -> Result<WebhookEndpoint> {
    let url = request.url.unwrap_or_default();
    let mut req = keygen_rs::webhook::endpoint::WebhookEndpointCreateRequest::new(url);

    if let Some(subs) = request.subscriptions {
        req = req.with_subscriptions(parse_webhook_events(subs));
    }
    if let Some(algo) = request.signature_algorithm {
        req = req.with_signature_algorithm(parse_signature_algorithm(&algo)?);
    }
    if let Some(env_id) = request.environment_id {
        req = req.with_environment_id(env_id);
    }

    keygen_rs::webhook::endpoint::WebhookEndpoint::create(req)
        .await
        .map(WebhookEndpoint::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_webhook_endpoints(
    options: Option<ListWebhookEndpointsOptions>,
) -> Result<Vec<WebhookEndpoint>> {
    let opts = options.map(
        |o| keygen_rs::webhook::endpoint::WebhookEndpointListOptions {
            limit: o.limit.map(|v| v as i32),
            page_size: o.page_size.map(|v| v as i32),
            page_number: o.page_number.map(|v| v as i32),
        },
    );
    keygen_rs::webhook::endpoint::WebhookEndpoint::list(opts.as_ref())
        .await
        .map(|list| list.into_iter().map(WebhookEndpoint::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_webhook_endpoint(id: String) -> Result<WebhookEndpoint> {
    keygen_rs::webhook::endpoint::WebhookEndpoint::get(&id)
        .await
        .map(WebhookEndpoint::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_webhook_endpoint(
    id: String,
    request: UpdateWebhookEndpointRequest,
) -> Result<WebhookEndpoint> {
    let ep = make_minimal_endpoint(id);
    let mut req = keygen_rs::webhook::endpoint::WebhookEndpointUpdateRequest::new();
    if let Some(url) = request.url {
        req = req.with_url(url);
    }
    if let Some(subs) = request.subscriptions {
        req = req.with_subscriptions(parse_webhook_events(subs));
    }
    if let Some(algo) = request.signature_algorithm {
        req = req.with_signature_algorithm(parse_signature_algorithm(&algo)?);
    }
    ep.update(req)
        .await
        .map(WebhookEndpoint::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_webhook_endpoint(id: String) -> Result<()> {
    let ep = make_minimal_endpoint(id);
    ep.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn list_webhook_events(
    options: Option<ListWebhookEventsOptions>,
) -> Result<Vec<WebhookEvent>> {
    let opts = options.map(|o| {
        let status = o.status.and_then(|s| {
            serde_json::from_value::<keygen_rs::webhook::event::WebhookEventStatus>(
                serde_json::Value::String(s),
            )
            .ok()
        });
        keygen_rs::webhook::event::WebhookEventListOptions {
            limit: o.limit.map(|v| v as i32),
            page_number: o.page_number.map(|v| v as i32),
            page_size: o.page_size.map(|v| v as i32),
            event_type: o.event_type,
            status,
        }
    });
    keygen_rs::webhook::event::WebhookEventRecord::list(opts.as_ref())
        .await
        .map(|list| list.into_iter().map(WebhookEvent::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_webhook_event(id: String) -> Result<WebhookEvent> {
    keygen_rs::webhook::event::WebhookEventRecord::get(&id)
        .await
        .map(WebhookEvent::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn retry_webhook_event(id: String) -> Result<WebhookEvent> {
    let evt = make_minimal_event_record(id);
    evt.retry()
        .await
        .map(WebhookEvent::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_webhook_event(id: String) -> Result<()> {
    let evt = make_minimal_event_record(id);
    evt.delete().await.map_err(to_napi_error)
}
