//! Request logs: the API requests made to an account.
//!
//! Request logs are available on Keygen Cloud and Keygen EE only. Keygen CE does not
//! have the endpoint. Use
//! [`Capabilities::supports_request_logs`](crate::Capabilities::supports_request_logs)
//! to check the server first. See <https://keygen.sh/docs/api/request-logs/>.
//!
//! A list returns the logs without their request and response bodies. Retrieve one log
//! with [`RequestLogService::get`] to read its bodies.

use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
use crate::{KeygenResponseData, ResourceIdentifier};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Reads the HTTP status, which the API reference shows as a string (`"422"`).
fn status_as_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    Ok(match Option::<Value>::deserialize(deserializer)? {
        Some(Value::String(status)) => Some(status),
        Some(Value::Number(status)) => Some(status.to_string()),
        _ => None,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RequestLogAttributes {
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default, deserialize_with = "status_as_string")]
    pub status: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub request_headers: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub request_body: Option<String>,
    #[serde(default)]
    pub response_headers: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub response_body: Option<String>,
    #[serde(default)]
    pub response_signature: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct RequestLogResponse {
    data: KeygenResponseData<RequestLogAttributes>,
}

/// One API request made to an account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequestLog {
    pub id: String,
    /// The request path, for example `/v1/accounts/<account>/machines`.
    pub url: Option<String>,
    pub method: Option<String>,
    /// The HTTP status of the response, for example `"422"`.
    pub status: Option<String>,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    /// A subset of the request headers.
    pub request_headers: Option<HashMap<String, Value>>,
    /// `None` in a list. Retrieve the log to read it.
    pub request_body: Option<String>,
    /// A subset of the response headers.
    pub response_headers: Option<HashMap<String, Value>>,
    /// `None` in a list. Retrieve the log to read it.
    pub response_body: Option<String>,
    pub response_signature: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    /// Who made the request, for example a user or a license.
    pub requestor: Option<ResourceIdentifier>,
    /// The resource the request was for.
    pub resource: Option<ResourceIdentifier>,
}

impl RequestLog {
    pub(crate) fn from(data: KeygenResponseData<RequestLogAttributes>) -> Self {
        let relationships = &data.relationships;
        let requestor = relationships.other_data("requestor").map(Into::into);
        let resource = relationships.other_data("resource").map(Into::into);
        let account_id = relationships.account_id();
        let environment_id = relationships.environment_id();
        let attributes = data.attributes;
        Self {
            id: data.id,
            url: attributes.url,
            method: attributes.method,
            status: attributes.status,
            user_agent: attributes.user_agent,
            ip: attributes.ip,
            request_headers: attributes.request_headers,
            request_body: attributes.request_body,
            response_headers: attributes.response_headers,
            response_body: attributes.response_body,
            response_signature: attributes.response_signature,
            created: attributes.created,
            updated: attributes.updated,
            account_id,
            environment_id,
            requestor,
            resource,
        }
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListRequestLogsOptions>,
    ) -> Result<Page<RequestLog>, Error> {
        let query = list_query(options)?;
        let response = client.get("request-logs", query.as_ref()).await?;
        let document: ListDocument<RequestLogAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(RequestLog::from))
    }

    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<RequestLog, Error> {
        let endpoint = format!("request-logs/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let response: RequestLogResponse = serde_json::from_value(response.body)?;
        Ok(RequestLog::from(response.data))
    }
}

/// Filters and pagination for [`RequestLogService::list`].
///
/// `resource` is not in the API reference, but keygen-api's `RequestLogsController`
/// accepts it.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListRequestLogsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Cursor returned by [`Page::next_cursor`].
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
    /// Start of the date range. Set it together with `date_end`. Keygen rounds the
    /// range to whole days and rejects ranges shorter than 1 or longer than 31 days.
    #[serde(rename = "date[start]", skip_serializing_if = "Option::is_none")]
    pub date_start: Option<DateTime<Utc>>,
    /// End of the date range. Set it together with `date_start`.
    #[serde(rename = "date[end]", skip_serializing_if = "Option::is_none")]
    pub date_end: Option<DateTime<Utc>>,
    /// Requestor type: `user`, `environment`, `product` or `license`. Use with
    /// `requestor_id`.
    #[serde(rename = "requestor[type]", skip_serializing_if = "Option::is_none")]
    pub requestor_type: Option<String>,
    /// Requestor ID. Use with `requestor_type`.
    #[serde(rename = "requestor[id]", skip_serializing_if = "Option::is_none")]
    pub requestor_id: Option<String>,
    /// Resource type, for example `license`. Use with `resource_id`.
    #[serde(rename = "resource[type]", skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// Resource ID. Use with `resource_type`.
    #[serde(rename = "resource[id]", skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// URL path, for example `/v1/accounts/<account>/licenses/actions/validate-key`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// IP address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    /// HTTP method, for example `POST`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// HTTP status, for example `400`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Request log operations bound to one [`KeygenClient`]. Keygen EE and Cloud only.
pub struct RequestLogService<'a> {
    client: &'a KeygenClient,
}

impl<'a> RequestLogService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    /// Lists request logs, most recent first. The request and response bodies are
    /// `None`; use [`RequestLogService::get`] to read them.
    pub async fn list(
        &self,
        options: Option<&ListRequestLogsOptions>,
    ) -> Result<Page<RequestLog>, Error> {
        RequestLog::list_with_client(&self.client.transport, options).await
    }

    /// Retrieves a request log with its request and response bodies.
    pub async fn get(&self, id: &str) -> Result<RequestLog, Error> {
        RequestLog::get_with_client(&self.client.transport, id).await
    }
}
