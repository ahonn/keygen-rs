//! Machine processes, used to limit application concurrency.
//!
//! A process belongs to a machine and must send heartbeat pings to stay alive.
//! See <https://keygen.sh/docs/api/processes/>.
//!
//! Spawning, pinging and killing a process also work with a license key. Listing,
//! retrieving and updating processes need the `token` feature, as for machines.

use crate::client::Client;
use crate::errors::Error;
#[cfg(feature = "token")]
use crate::insert_optional;
use crate::keygen_client::KeygenClient;
#[cfg(feature = "token")]
use crate::pagination::{list_query, ListDocument, Page};
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// The heartbeat status of a process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ProcessStatus {
    Alive,
    Dead,
    Resurrected,
    /// A status this SDK does not know.
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProcessAttributes {
    pub pid: String,
    pub status: ProcessStatus,
    #[serde(default)]
    pub last_heartbeat: Option<DateTime<Utc>>,
    #[serde(default)]
    pub next_heartbeat: Option<DateTime<Utc>>,
    #[serde(default)]
    pub interval: Option<i64>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProcessResponse {
    data: KeygenResponseData<ProcessAttributes>,
}

/// A process running on a machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Process {
    pub id: String,
    /// The process ID given at spawn. Unique per machine.
    pub pid: String,
    pub status: ProcessStatus,
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// The time by which the process must send its next heartbeat ping.
    pub next_heartbeat: Option<DateTime<Utc>>,
    /// The heartbeat interval in seconds, inherited from the license's policy.
    pub interval: Option<i64>,
    pub metadata: Option<HashMap<String, Value>>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub machine_id: Option<String>,
}

impl Process {
    pub(crate) fn from(data: KeygenResponseData<ProcessAttributes>) -> Self {
        Self {
            machine_id: data.relationships.other_data("machine").map(|d| d.id),
            account_id: data.relationships.account_id(),
            environment_id: data.relationships.environment_id(),
            product_id: data.relationships.product_id(),
            license_id: data.relationships.license_id(),
            id: data.id,
            pid: data.attributes.pid,
            status: data.attributes.status,
            last_heartbeat: data.attributes.last_heartbeat,
            next_heartbeat: data.attributes.next_heartbeat,
            interval: data.attributes.interval,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
        }
    }

    fn from_body(body: Value) -> Result<Self, Error> {
        let response: ProcessResponse = serde_json::from_value(body)?;
        Ok(Self::from(response.data))
    }

    pub(crate) async fn spawn_with_client(
        client: &Client,
        request: &SpawnProcessRequest,
    ) -> Result<Process, Error> {
        let mut attributes = serde_json::Map::new();
        attributes.insert("pid".to_string(), json!(request.pid));
        if let Some(metadata) = &request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }
        let body = json!({
            "data": {
                "type": "processes",
                "attributes": attributes,
                "relationships": {
                    "machine": { "data": { "type": "machines", "id": request.machine_id } }
                }
            }
        });
        let response = client.post("processes", Some(&body), None::<&()>).await?;
        Self::from_body(response.body)
    }

    pub(crate) async fn ping_with_client(client: &Client, id: &str) -> Result<Process, Error> {
        let endpoint = format!("processes/{id}/actions/ping");
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        Self::from_body(response.body)
    }

    pub(crate) async fn kill_with_client(client: &Client, id: &str) -> Result<(), Error> {
        let endpoint = format!("processes/{id}");
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    #[cfg(feature = "token")]
    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListProcessesOptions>,
    ) -> Result<Page<Process>, Error> {
        let query = list_query(options)?;
        let response = client.get("processes", query.as_ref()).await?;
        let document: ListDocument<ProcessAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Process::from))
    }

    #[cfg(feature = "token")]
    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Process, Error> {
        let endpoint = format!("processes/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        Self::from_body(response.body)
    }

    #[cfg(feature = "token")]
    pub(crate) async fn update_with_client(
        client: &Client,
        id: &str,
        request: UpdateProcessRequest,
    ) -> Result<Process, Error> {
        let mut attributes = serde_json::Map::new();
        insert_optional(&mut attributes, "metadata", request.metadata)?;
        let body = json!({ "data": { "type": "processes", "attributes": attributes } });
        let endpoint = format!("processes/{id}");
        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        Self::from_body(response.body)
    }
}

/// The attributes of a new process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpawnProcessRequest {
    /// The process ID. Any string that is unique for the machine.
    pub pid: String,
    /// The machine the process runs on.
    pub machine_id: String,
    pub metadata: Option<HashMap<String, Value>>,
}

/// The attributes of a process to change. Needs an admin, environment or product token.
#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateProcessRequest {
    pub metadata: Option<HashMap<String, Value>>,
}

/// Filters and pagination for [`ProcessService::list`].
#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListProcessesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Cursor returned by [`Page::next_cursor`].
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
    /// Machine ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    /// License ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Owner (user) ID of the license or machine.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// User ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Product ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
}

/// Process operations bound to one [`KeygenClient`].
pub struct ProcessService<'a> {
    client: &'a KeygenClient,
}

impl<'a> ProcessService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    /// Spawns a process. The process must then send heartbeat pings with
    /// [`ProcessService::ping`] to stay alive.
    pub async fn spawn(&self, request: &SpawnProcessRequest) -> Result<Process, Error> {
        Process::spawn_with_client(&self.client.transport, request).await
    }

    /// Sends a heartbeat ping for a process.
    pub async fn ping(&self, id: &str) -> Result<Process, Error> {
        Process::ping_with_client(&self.client.transport, id).await
    }

    /// Kills (deletes) a process.
    pub async fn kill(&self, id: &str) -> Result<(), Error> {
        Process::kill_with_client(&self.client.transport, id).await
    }

    #[cfg(feature = "token")]
    pub async fn list(
        &self,
        options: Option<&ListProcessesOptions>,
    ) -> Result<Page<Process>, Error> {
        Process::list_with_client(&self.client.transport, options).await
    }

    #[cfg(feature = "token")]
    pub async fn get(&self, id: &str) -> Result<Process, Error> {
        Process::get_with_client(&self.client.transport, id).await
    }

    #[cfg(feature = "token")]
    pub async fn update(&self, id: &str, request: UpdateProcessRequest) -> Result<Process, Error> {
        Process::update_with_client(&self.client.transport, id, request).await
    }
}
