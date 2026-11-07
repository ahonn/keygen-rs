//! Machine activation and management.
//!
//! This module provides functionality for activating machines against licenses,
//! managing machine heartbeats, and checking out machine files for offline use.

use crate::certificate::CertificateFileResponse;
use crate::client::{Client, ClientOptions, Response};
use crate::config::get_config;
use crate::config::KeygenConfig;
use crate::errors::Error;
#[cfg(feature = "token")]
use crate::insert_optional;
use crate::keygen_client::KeygenClient;
use crate::machine_file::MachineFile;
#[cfg(feature = "token")]
use crate::pagination::{cursor_pagination_query, validate_page_size, ListDocument, Page};
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
#[cfg(not(target_arch = "wasm32"))]
use futures::future::{BoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc;

/// Heartbeat status as returned by the Keygen API
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeartbeatStatus {
    Alive,
    Dead,
    NotStarted,
    Resurrected,
}

impl HeartbeatStatus {
    /// Parses a HeartbeatStatus from a string, returning None for unknown values
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ALIVE" => Some(Self::Alive),
            "DEAD" => Some(Self::Dead),
            "NOT_STARTED" => Some(Self::NotStarted),
            "RESURRECTED" => Some(Self::Resurrected),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineAttributes {
    pub fingerprint: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    pub metadata: Option<HashMap<String, Value>>,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: bool,
    #[serde(rename = "heartbeatStatus")]
    pub heartbeat_status: String,
    #[serde(rename = "heartbeatDuration")]
    pub heartbeat_duration: Option<i32>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineResponse {
    pub data: KeygenResponseData<MachineAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachinesResponse {
    pub data: Vec<KeygenResponseData<MachineAttributes>>,
    pub meta: Option<Value>,
    pub links: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Machine {
    pub id: String,
    pub fingerprint: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    #[cfg_attr(
        feature = "specta",
        specta(type = Option<crate::specta_support::JsonMetadata>)
    )]
    pub metadata: Option<HashMap<String, Value>>,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: bool,
    #[serde(rename = "heartbeatStatus")]
    pub heartbeat_status: String,
    #[serde(rename = "heartbeatDuration")]
    pub heartbeat_duration: Option<i32>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
    #[serde(skip)]
    pub config: Option<Arc<KeygenConfig>>,
}

#[derive(Debug, Clone, Default)]
pub struct MachineCheckoutOpts {
    pub ttl: Option<i64>,
    pub include: Option<Vec<String>>,
}

impl MachineCheckoutOpts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_ttl(ttl: i64) -> Self {
        Self {
            ttl: Some(ttl),
            ..Self::default()
        }
    }

    pub fn with_include(include: Vec<String>) -> Self {
        Self {
            include: Some(include),
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MachineListFilters {
    pub license: Option<String>,
    pub user: Option<String>,
    pub platform: Option<String>,
    pub name: Option<String>,
    pub fingerprint: Option<String>,
    pub ip: Option<String>,
    pub hostname: Option<String>,
    pub product: Option<String>,
    pub owner: Option<String>,
    pub group: Option<String>,
    pub policy: Option<String>,
    pub key: Option<String>,
    pub metadata: Option<HashMap<String, Value>>,
    pub page_number: Option<i32>,
    pub page_size: Option<i32>,
    pub limit: Option<i32>,
    /// Cursor returned by [`Page::next_cursor`]. Ignored when `page_number` is set.
    pub page_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineCreateRequest {
    pub fingerprint: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    pub metadata: Option<HashMap<String, Value>>,
    pub license_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineUpdateRequest {
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub ip: Option<String>,
    pub cores: Option<i32>,
    pub metadata: Option<HashMap<String, Value>>,
}

impl Machine {
    pub(crate) fn from(data: KeygenResponseData<MachineAttributes>) -> Machine {
        Machine {
            id: data.id,
            fingerprint: data.attributes.fingerprint,
            name: data.attributes.name,
            platform: data.attributes.platform,
            hostname: data.attributes.hostname,
            ip: data.attributes.ip,
            cores: data.attributes.cores,
            metadata: data.attributes.metadata,
            require_heartbeat: data.attributes.require_heartbeat,
            heartbeat_status: data.attributes.heartbeat_status,
            heartbeat_duration: data.attributes.heartbeat_duration,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data.relationships.account_id(),
            environment_id: data.relationships.environment_id(),
            product_id: data.relationships.product_id(),
            license_id: data.relationships.license_id(),
            owner_id: data.relationships.owner_id(),
            group_id: data.relationships.group_id(),
            config: None,
        }
    }

    /// Associates a configuration with this Machine
    pub fn with_config(mut self, config: KeygenConfig) -> Self {
        self.config = Some(Arc::new(config));
        self
    }

    pub(crate) fn with_config_arc(mut self, config: Arc<KeygenConfig>) -> Self {
        self.config = Some(config);
        self
    }

    /// Gives a machine returned by a call on `self` the same configuration.
    #[cfg(feature = "token")]
    fn inherit_config(&self, mut machine: Machine) -> Machine {
        machine.config = self.config.clone();
        machine
    }

    /// Gets a client for this machine, using the associated config or global config
    fn get_client(&self) -> Result<Client, Error> {
        let config = if let Some(ref cfg) = self.config {
            cfg.as_ref().clone()
        } else {
            get_config()?
        };
        Client::new(ClientOptions::from(config))
    }

    pub async fn deactivate(&self) -> Result<(), Error> {
        Self::deactivate_with_client(&self.get_client()?, &self.id).await
    }

    pub(crate) async fn deactivate_with_client(client: &Client, id: &str) -> Result<(), Error> {
        client
            .delete::<(), serde_json::Value>(&format!("machines/{id}"), None::<&()>)
            .await?;
        Ok(())
    }

    pub async fn checkout(&self, options: &MachineCheckoutOpts) -> Result<MachineFile, Error> {
        Self::checkout_with_client(&self.get_client()?, &self.id, options).await
    }

    pub(crate) async fn checkout_with_client(
        client: &Client,
        id: &str,
        options: &MachineCheckoutOpts,
    ) -> Result<MachineFile, Error> {
        let mut query = json!({
            "encrypt": 1
        });

        if let Some(ttl) = options.ttl {
            query["ttl"] = ttl.into();
        }

        if let Some(ref include) = options.include {
            query["include"] = json!(include.join(","));
        } else {
            query["include"] = "license.entitlements".into();
        }

        let response = client
            .post(
                &format!("machines/{id}/actions/check-out"),
                None::<&()>,
                Some(&query),
            )
            .await?;

        let machine_file_response: CertificateFileResponse = serde_json::from_value(response.body)?;
        let machine_file = MachineFile::from(machine_file_response.data);
        Ok(machine_file)
    }

    pub async fn ping(&self) -> Result<Machine, Error> {
        let config = self.config.as_ref().ok_or(Error::MissingConfiguration)?;
        let machine = Self::ping_with_client(&self.get_client()?, &self.id).await?;
        Ok(machine.with_config_arc(Arc::clone(config)))
    }

    pub(crate) async fn ping_with_client(client: &Client, id: &str) -> Result<Machine, Error> {
        let response: Response<MachineResponse> = client
            .post(
                &format!("machines/{id}/actions/ping"),
                None::<&()>,
                None::<&()>,
            )
            .await?;
        Ok(Machine::from(response.body.data))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn monitor(
        self: Arc<Self>,
        heartbeat_interval: Duration,
        tx: Option<mpsc::Sender<Result<Machine, Error>>>,
        mut cancel_rx: Option<mpsc::Receiver<()>>,
    ) -> BoxFuture<'static, ()> {
        async move {
            async fn send(
                tx: &Option<mpsc::Sender<Result<Machine, Error>>>,
                result: Result<Machine, Error>,
            ) {
                if let Some(tx) = tx {
                    let _ = tx.send(result).await;
                }
            }

            let mut interval = tokio::time::interval(heartbeat_interval);
            interval.tick().await;

            send(&tx, self.ping().await).await;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        send(&tx, self.ping().await).await;
                    }
                    _ = async {
                        if let Some(ref mut rx) = cancel_rx {
                            rx.recv().await
                        } else {
                            std::future::pending::<Option<()>>().await
                        }
                    } => {
                        break;
                    }
                }
            }
        }
        .boxed()
    }

    /// Create a new machine
    #[cfg(feature = "token")]
    pub async fn create(request: MachineCreateRequest) -> Result<Machine, Error> {
        Self::create_with_client(&Client::from_global_config()?, request).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn create_with_client(
        client: &Client,
        request: MachineCreateRequest,
    ) -> Result<Machine, Error> {
        let mut attributes = serde_json::Map::new();
        attributes.insert("fingerprint".to_string(), json!(request.fingerprint));

        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "platform", request.platform)?;
        insert_optional(&mut attributes, "hostname", request.hostname)?;
        insert_optional(&mut attributes, "ip", request.ip)?;
        insert_optional(&mut attributes, "cores", request.cores)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = json!({
            "data": {
                "type": "machines",
                "attributes": attributes,
                "relationships": {
                    "license": {
                        "data": {
                            "type": "licenses",
                            "id": request.license_id
                        }
                    }
                }
            }
        });

        let response = client.post("machines", Some(&body), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }

    /// List machines with optional filters
    #[cfg(feature = "token")]
    pub async fn list(filters: Option<MachineListFilters>) -> Result<Vec<Machine>, Error> {
        let page = Self::list_with_client(&Client::from_global_config()?, filters.as_ref()).await?;
        Ok(page.into_items())
    }

    #[cfg(feature = "token")]
    pub(crate) async fn list_with_client(
        client: &Client,
        filters: Option<&MachineListFilters>,
    ) -> Result<Page<Machine>, Error> {
        let mut query_params = Vec::new();
        if let Some(filters) = filters {
            let mut push = |key: &str, value: &Option<String>| {
                if let Some(value) = value {
                    query_params.push((key.to_string(), value.clone()));
                }
            };
            push("license", &filters.license);
            push("user", &filters.user);
            push("platform", &filters.platform);
            push("name", &filters.name);
            push("fingerprint", &filters.fingerprint);
            push("ip", &filters.ip);
            push("hostname", &filters.hostname);
            push("product", &filters.product);
            push("owner", &filters.owner);
            push("group", &filters.group);
            push("policy", &filters.policy);
            push("key", &filters.key);
            if let Some(metadata) = &filters.metadata {
                for (key, value) in metadata {
                    query_params.push((format!("metadata[{key}]"), value.to_string()));
                }
            }

            if let Some(page_number) = filters.page_number {
                // Page-number pagination, kept for compatibility.
                query_params.push(("page[number]".to_string(), page_number.to_string()));
                if let Some(page_size) = filters.page_size {
                    validate_page_size(page_size.into())?;
                    query_params.push(("page[size]".to_string(), page_size.to_string()));
                }
                if let Some(limit) = filters.limit {
                    query_params.push(("limit".to_string(), limit.to_string()));
                }
            } else if let Value::Object(pagination) = cursor_pagination_query(
                filters.limit,
                filters.page_size,
                filters.page_cursor.as_deref(),
                None,
            )? {
                for (key, value) in pagination {
                    let value = match value {
                        Value::String(value) => value,
                        value => value.to_string(),
                    };
                    query_params.push((key, value));
                }
            }
        }

        let query = if query_params.is_empty() {
            None
        } else {
            Some(
                query_params
                    .into_iter()
                    .collect::<HashMap<String, String>>(),
            )
        };

        let response = client.get("machines", query.as_ref()).await?;
        let document: ListDocument<MachineAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Machine::from))
    }

    /// Get a machine by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<Machine, Error> {
        Self::get_with_client(&Client::from_global_config()?, id).await
    }

    #[cfg(feature = "token")]
    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Machine, Error> {
        let endpoint = format!("machines/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }

    /// Update a machine
    #[cfg(feature = "token")]
    pub async fn update(&self, request: MachineUpdateRequest) -> Result<Machine, Error> {
        let machine = Self::update_with_client(&self.get_client()?, &self.id, request).await?;
        Ok(self.inherit_config(machine))
    }

    #[cfg(feature = "token")]
    pub(crate) async fn update_with_client(
        client: &Client,
        id: &str,
        request: MachineUpdateRequest,
    ) -> Result<Machine, Error> {
        let endpoint = format!("machines/{id}");

        let mut attributes = serde_json::Map::new();
        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "platform", request.platform)?;
        insert_optional(&mut attributes, "hostname", request.hostname)?;
        insert_optional(&mut attributes, "ip", request.ip)?;
        insert_optional(&mut attributes, "cores", request.cores)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = json!({
            "data": {
                "type": "machines",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }

    /// Reset machine heartbeat
    #[cfg(feature = "token")]
    pub async fn reset(&self) -> Result<Machine, Error> {
        let machine = Self::reset_with_client(&self.get_client()?, &self.id).await?;
        Ok(self.inherit_config(machine))
    }

    #[cfg(feature = "token")]
    pub(crate) async fn reset_with_client(client: &Client, id: &str) -> Result<Machine, Error> {
        let endpoint = format!("machines/{id}/actions/reset");
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }

    /// Change the machine owner.
    #[cfg(feature = "token")]
    pub async fn change_owner(&self, owner_id: &str) -> Result<Machine, Error> {
        let machine =
            Self::change_owner_with_client(&self.get_client()?, &self.id, owner_id).await?;
        Ok(self.inherit_config(machine))
    }

    #[cfg(feature = "token")]
    pub(crate) async fn change_owner_with_client(
        client: &Client,
        id: &str,
        owner_id: &str,
    ) -> Result<Machine, Error> {
        let endpoint = format!("machines/{id}/owner");
        let body = json!({
            "data": {
                "type": "users",
                "id": owner_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }

    /// Change the machine group.
    #[cfg(feature = "token")]
    pub async fn change_group(&self, group_id: &str) -> Result<Machine, Error> {
        let machine =
            Self::change_group_with_client(&self.get_client()?, &self.id, group_id).await?;
        Ok(self.inherit_config(machine))
    }

    #[cfg(feature = "token")]
    pub(crate) async fn change_group_with_client(
        client: &Client,
        id: &str,
        group_id: &str,
    ) -> Result<Machine, Error> {
        let endpoint = format!("machines/{id}/group");
        let body = json!({
            "data": {
                "type": "groups",
                "id": group_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }
}

/// Machine operations bound to one [`KeygenClient`].
///
/// Every machine returned by this service carries the client's configuration, so
/// follow-up calls such as [`Machine::deactivate`] or [`Machine::ping`] use the
/// same account and environment.
pub struct MachineService<'a> {
    client: &'a KeygenClient,
}

impl<'a> MachineService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    fn bind(&self, machine: Machine) -> Machine {
        machine.with_config_arc(self.client.config_arc())
    }

    pub async fn deactivate(&self, id: &str) -> Result<(), Error> {
        Machine::deactivate_with_client(&self.client.transport, id).await
    }

    pub async fn checkout(
        &self,
        id: &str,
        options: &MachineCheckoutOpts,
    ) -> Result<MachineFile, Error> {
        Machine::checkout_with_client(&self.client.transport, id, options).await
    }

    pub async fn ping(&self, id: &str) -> Result<Machine, Error> {
        let machine = Machine::ping_with_client(&self.client.transport, id).await?;
        Ok(self.bind(machine))
    }

    #[cfg(feature = "token")]
    pub async fn create(&self, request: MachineCreateRequest) -> Result<Machine, Error> {
        let machine = Machine::create_with_client(&self.client.transport, request).await?;
        Ok(self.bind(machine))
    }

    #[cfg(feature = "token")]
    pub async fn list(&self, filters: Option<&MachineListFilters>) -> Result<Page<Machine>, Error> {
        let page = Machine::list_with_client(&self.client.transport, filters).await?;
        Ok(page.map(|machine| self.bind(machine)))
    }

    #[cfg(feature = "token")]
    pub async fn get(&self, id: &str) -> Result<Machine, Error> {
        let machine = Machine::get_with_client(&self.client.transport, id).await?;
        Ok(self.bind(machine))
    }

    #[cfg(feature = "token")]
    pub async fn update(&self, id: &str, request: MachineUpdateRequest) -> Result<Machine, Error> {
        let machine = Machine::update_with_client(&self.client.transport, id, request).await?;
        Ok(self.bind(machine))
    }

    #[cfg(feature = "token")]
    pub async fn reset(&self, id: &str) -> Result<Machine, Error> {
        let machine = Machine::reset_with_client(&self.client.transport, id).await?;
        Ok(self.bind(machine))
    }

    #[cfg(feature = "token")]
    pub async fn change_owner(&self, id: &str, owner_id: &str) -> Result<Machine, Error> {
        let machine =
            Machine::change_owner_with_client(&self.client.transport, id, owner_id).await?;
        Ok(self.bind(machine))
    }

    #[cfg(feature = "token")]
    pub async fn change_group(&self, id: &str, group_id: &str) -> Result<Machine, Error> {
        let machine =
            Machine::change_group_with_client(&self.client.transport, id, group_id).await?;
        Ok(self.bind(machine))
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
    fn test_machine_relationships() {
        // Test that all relationship IDs are properly extracted
        let machine_data = KeygenResponseData {
            id: "test-machine-id".to_string(),
            r#type: "machines".to_string(),
            attributes: MachineAttributes {
                fingerprint: "test-fingerprint".to_string(),
                name: Some("Test Machine".to_string()),
                platform: Some("linux".to_string()),
                hostname: Some("test-host".to_string()),
                ip: Some("192.168.1.1".to_string()),
                cores: Some(8),
                metadata: Some(HashMap::new()),
                require_heartbeat: true,
                heartbeat_status: "ALIVE".to_string(),
                heartbeat_duration: Some(3600),
                created: Utc::now(),
                updated: Utc::now(),
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
                product: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    }),
                    links: None,
                }),
                group: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "groups".to_string(),
                        id: "test-group-id".to_string(),
                    }),
                    links: None,
                }),
                owner: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "users".to_string(),
                        id: "test-owner-id".to_string(),
                    }),
                    links: None,
                }),
                users: None,
                machines: None,
                environment: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "environments".to_string(),
                        id: "test-environment-id".to_string(),
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
                release: None,
                other: HashMap::new(),
            },
        };

        let machine = Machine::from(machine_data);

        assert_eq!(machine.account_id, Some("test-account-id".to_string()));
        assert_eq!(
            machine.environment_id,
            Some("test-environment-id".to_string())
        );
        assert_eq!(machine.product_id, Some("test-product-id".to_string()));
        assert_eq!(machine.license_id, Some("test-license-id".to_string()));
        assert_eq!(machine.owner_id, Some("test-owner-id".to_string()));
        assert_eq!(machine.group_id, Some("test-group-id".to_string()));
        assert_eq!(machine.id, "test-machine-id");
        assert_eq!(machine.fingerprint, "test-fingerprint");
    }

    #[test]
    fn test_machine_without_relationships() {
        // Test that all relationship IDs are None when no relationships exist
        let machine_data = KeygenResponseData {
            id: "test-machine-id".to_string(),
            r#type: "machines".to_string(),
            attributes: MachineAttributes {
                fingerprint: "test-fingerprint".to_string(),
                name: Some("Test Machine".to_string()),
                platform: Some("linux".to_string()),
                hostname: Some("test-host".to_string()),
                ip: Some("192.168.1.1".to_string()),
                cores: Some(8),
                metadata: Some(HashMap::new()),
                require_heartbeat: true,
                heartbeat_status: "ALIVE".to_string(),
                heartbeat_duration: Some(3600),
                created: Utc::now(),
                updated: Utc::now(),
            },
            relationships: KeygenRelationships {
                policy: None,
                account: None,
                product: None,
                group: None,
                owner: None,
                users: None,
                machines: None,
                environment: None,
                license: None,
                release: None,
                other: HashMap::new(),
            },
        };

        let machine = Machine::from(machine_data);

        assert_eq!(machine.account_id, None);
        assert_eq!(machine.environment_id, None);
        assert_eq!(machine.product_id, None);
        assert_eq!(machine.license_id, None);
        assert_eq!(machine.owner_id, None);
        assert_eq!(machine.group_id, None);
    }

    #[test]
    fn test_heartbeat_status_parse() {
        assert_eq!(
            HeartbeatStatus::parse("ALIVE"),
            Some(HeartbeatStatus::Alive)
        );
        assert_eq!(HeartbeatStatus::parse("DEAD"), Some(HeartbeatStatus::Dead));
        assert_eq!(
            HeartbeatStatus::parse("NOT_STARTED"),
            Some(HeartbeatStatus::NotStarted)
        );
        assert_eq!(
            HeartbeatStatus::parse("RESURRECTED"),
            Some(HeartbeatStatus::Resurrected)
        );
        assert_eq!(HeartbeatStatus::parse("UNKNOWN"), None);
    }

    #[test]
    fn test_heartbeat_status_parse_case_insensitive() {
        assert_eq!(
            HeartbeatStatus::parse("alive"),
            Some(HeartbeatStatus::Alive)
        );
        assert_eq!(HeartbeatStatus::parse("dead"), Some(HeartbeatStatus::Dead));
        assert_eq!(
            HeartbeatStatus::parse("not_started"),
            Some(HeartbeatStatus::NotStarted)
        );
        assert_eq!(
            HeartbeatStatus::parse("resurrected"),
            Some(HeartbeatStatus::Resurrected)
        );
    }

    #[test]
    fn test_heartbeat_status_serialize() {
        assert_eq!(
            serde_json::to_string(&HeartbeatStatus::Alive).unwrap(),
            "\"ALIVE\""
        );
        assert_eq!(
            serde_json::to_string(&HeartbeatStatus::Dead).unwrap(),
            "\"DEAD\""
        );
        assert_eq!(
            serde_json::to_string(&HeartbeatStatus::NotStarted).unwrap(),
            "\"NOT_STARTED\""
        );
        assert_eq!(
            serde_json::to_string(&HeartbeatStatus::Resurrected).unwrap(),
            "\"RESURRECTED\""
        );
    }

    #[test]
    fn test_heartbeat_status_deserialize() {
        assert_eq!(
            serde_json::from_str::<HeartbeatStatus>("\"ALIVE\"").unwrap(),
            HeartbeatStatus::Alive
        );
        assert_eq!(
            serde_json::from_str::<HeartbeatStatus>("\"DEAD\"").unwrap(),
            HeartbeatStatus::Dead
        );
        assert_eq!(
            serde_json::from_str::<HeartbeatStatus>("\"NOT_STARTED\"").unwrap(),
            HeartbeatStatus::NotStarted
        );
        assert_eq!(
            serde_json::from_str::<HeartbeatStatus>("\"RESURRECTED\"").unwrap(),
            HeartbeatStatus::Resurrected
        );
    }
}
