use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

fn parse_signature_algorithm(
    s: &str,
) -> Result<keygen_rs::webhook::endpoint::SignatureAlgorithm, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid signature algorithm: {e}")))
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

// -- WebhookEndpoint CRUD --

#[wasm_bindgen(js_name = "createWebhookEndpoint")]
pub async fn create_webhook_endpoint(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        url: Option<String>,
        subscriptions: Option<Vec<String>>,
        signature_algorithm: Option<String>,
        environment_id: Option<String>,
    }

    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let url = req.url.unwrap_or_default();
    let mut r = keygen_rs::webhook::endpoint::WebhookEndpointCreateRequest::new(url);

    if let Some(subs) = req.subscriptions {
        r = r.with_subscriptions(parse_webhook_events(subs));
    }
    if let Some(algo) = req.signature_algorithm {
        r = r.with_signature_algorithm(parse_signature_algorithm(&algo)?);
    }
    if let Some(env_id) = req.environment_id {
        r = r.with_environment_id(env_id);
    }

    let endpoint = keygen_rs::webhook::endpoint::WebhookEndpoint::create(r)
        .await
        .map(WebhookEndpoint::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&endpoint).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listWebhookEndpoints")]
pub async fn list_webhook_endpoints(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
    }

    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let keygen_opts = opts.map(
        |o| keygen_rs::webhook::endpoint::WebhookEndpointListOptions {
            limit: o.limit.map(|v| v as i32),
            page_size: o.page_size.map(|v| v as i32),
            page_number: o.page_number.map(|v| v as i32),
        },
    );

    let endpoints: Vec<WebhookEndpoint> =
        keygen_rs::webhook::endpoint::WebhookEndpoint::list(keygen_opts.as_ref())
            .await
            .map(|list| list.into_iter().map(WebhookEndpoint::from).collect())
            .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&endpoints).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getWebhookEndpoint")]
pub async fn get_webhook_endpoint(id: String) -> Result<JsValue, JsError> {
    let endpoint = keygen_rs::webhook::endpoint::WebhookEndpoint::get(&id)
        .await
        .map(WebhookEndpoint::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&endpoint).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateWebhookEndpoint")]
pub async fn update_webhook_endpoint(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        url: Option<String>,
        subscriptions: Option<Vec<String>>,
        signature_algorithm: Option<String>,
    }

    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let ep = make_minimal_endpoint(id);
    let mut r = keygen_rs::webhook::endpoint::WebhookEndpointUpdateRequest::new();
    if let Some(url) = req.url {
        r = r.with_url(url);
    }
    if let Some(subs) = req.subscriptions {
        r = r.with_subscriptions(parse_webhook_events(subs));
    }
    if let Some(algo) = req.signature_algorithm {
        r = r.with_signature_algorithm(parse_signature_algorithm(&algo)?);
    }

    let endpoint = ep
        .update(r)
        .await
        .map(WebhookEndpoint::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&endpoint).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteWebhookEndpoint")]
pub async fn delete_webhook_endpoint(id: String) -> Result<(), JsError> {
    let ep = make_minimal_endpoint(id);
    ep.delete().await.map_err(to_js_error)
}

// -- WebhookEvent operations --

#[wasm_bindgen(js_name = "listWebhookEvents")]
pub async fn list_webhook_events(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
        event_type: Option<String>,
        status: Option<String>,
    }

    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let keygen_opts = opts.map(|o| {
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

    let events: Vec<WebhookEvent> =
        keygen_rs::webhook::event::WebhookEventRecord::list(keygen_opts.as_ref())
            .await
            .map(|list| list.into_iter().map(WebhookEvent::from).collect())
            .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&events).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getWebhookEvent")]
pub async fn get_webhook_event(id: String) -> Result<JsValue, JsError> {
    let event = keygen_rs::webhook::event::WebhookEventRecord::get(&id)
        .await
        .map(WebhookEvent::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&event).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "retryWebhookEvent")]
pub async fn retry_webhook_event(id: String) -> Result<JsValue, JsError> {
    let evt = make_minimal_event_record(id);
    let event = evt
        .retry()
        .await
        .map(WebhookEvent::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&event).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteWebhookEvent")]
pub async fn delete_webhook_event(id: String) -> Result<(), JsError> {
    let evt = make_minimal_event_record(id);
    evt.delete().await.map_err(to_js_error)
}
