//! Event logs: an audit trail of the events on an account.
//!
//! Event logs are available on Keygen Cloud and Keygen EE only. Keygen CE does not have
//! the endpoint. Use [`Capabilities::supports_event_logs`](crate::Capabilities::supports_event_logs)
//! to check the server first. See <https://keygen.sh/docs/api/event-logs/>.

use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
use crate::{KeygenResponseData, ResourceIdentifier};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EventLogAttributes {
    pub event: String,
    #[serde(default)]
    pub metadata: Option<Map<String, Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct EventLogResponse {
    data: KeygenResponseData<EventLogAttributes>,
}

/// One event on an account, for example `license.updated`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventLog {
    pub id: String,
    /// The event type, for example `license.validation.succeeded`.
    pub event: String,
    /// Event details, such as a validation code or a diff of changed attributes.
    /// Keygen documents the content as subject to change.
    pub metadata: Option<Map<String, Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    /// The ID of the request log of the request that triggered the event.
    pub request_id: Option<String>,
    /// Who triggered the event, for example a user or a license.
    pub whodunnit: Option<ResourceIdentifier>,
    /// The resource the event is for.
    pub resource: Option<ResourceIdentifier>,
}

impl EventLog {
    pub(crate) fn from(data: KeygenResponseData<EventLogAttributes>) -> Self {
        let relationships = &data.relationships;
        Self {
            account_id: relationships.account_id(),
            environment_id: relationships.environment_id(),
            request_id: relationships.other_data("request").map(|d| d.id),
            whodunnit: relationships.other_data("whodunnit").map(Into::into),
            resource: relationships.other_data("resource").map(Into::into),
            id: data.id,
            event: data.attributes.event,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListEventLogsOptions>,
    ) -> Result<Page<EventLog>, Error> {
        let query = list_query(options)?;
        let response = client.get("event-logs", query.as_ref()).await?;
        let document: ListDocument<EventLogAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(EventLog::from))
    }

    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<EventLog, Error> {
        let endpoint = format!("event-logs/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let response: EventLogResponse = serde_json::from_value(response.body)?;
        Ok(EventLog::from(response.data))
    }
}

/// Filters and pagination for [`EventLogService::list`].
///
/// `date`, `resource` and pagination are in the API reference. `whodunnit`, `request`
/// and `event` are not, but keygen-api's `EventLogsController` accepts them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListEventLogsOptions {
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
    /// Resource type, for example `license` or `licenses`. Use with `resource_id`.
    #[serde(rename = "resource[type]", skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<String>,
    /// Resource ID. Use with `resource_type`.
    #[serde(rename = "resource[id]", skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Type of who triggered the event, for example `user`. Use with `whodunnit_id`.
    #[serde(rename = "whodunnit[type]", skip_serializing_if = "Option::is_none")]
    pub whodunnit_type: Option<String>,
    /// ID of who triggered the event. Use with `whodunnit_type`.
    #[serde(rename = "whodunnit[id]", skip_serializing_if = "Option::is_none")]
    pub whodunnit_id: Option<String>,
    /// Request log ID.
    #[serde(rename = "request", skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Event type, for example `license.created`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
}

/// Event log operations bound to one [`KeygenClient`]. Keygen EE and Cloud only.
pub struct EventLogService<'a> {
    client: &'a KeygenClient,
}

impl<'a> EventLogService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    /// Lists event logs, most recent first.
    pub async fn list(
        &self,
        options: Option<&ListEventLogsOptions>,
    ) -> Result<Page<EventLog>, Error> {
        EventLog::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<EventLog, Error> {
        EventLog::get_with_client(&self.client.transport, id).await
    }
}
