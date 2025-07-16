use crate::certificate::CartificateFileResponse;
use crate::client::{Client, Response};
use crate::errors::Error;
use crate::machine_file::MachineFile;
use crate::KeygenResponseData;
use chrono::{DateTime, Utc};
use futures::future::{BoxFuture, FutureExt};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
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
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub owner_id: Option<String>,
    pub group_id: Option<String>,
}

pub struct MachineCheckoutOpts {
    pub ttl: Option<i64>,
    pub include: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineListFilters {
    pub license: Option<String>,
    pub user: Option<String>,
    pub platform: Option<String>,
    pub name: Option<String>,
    pub fingerprint: Option<String>,
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
            account_id: data.relationships.account.as_ref().and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
            environment_id: data.relationships.environment.as_ref().and_then(|e| e.data.as_ref().map(|d| d.id.clone())),
            product_id: data.relationships.product.as_ref().and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            license_id: data.relationships.license.as_ref().and_then(|l| l.data.as_ref().map(|d| d.id.clone())),
            owner_id: data.relationships.owner.as_ref().and_then(|o| o.data.as_ref().map(|d| d.id.clone())),
            group_id: data.relationships.group.as_ref().and_then(|g| g.data.as_ref().map(|d| d.id.clone())),
        }
    }

    pub async fn deactivate(&self) -> Result<(), Error> {
        let client = Client::default();
        let _response = client
            .delete::<(), serde_json::Value>(&format!("machines/{}", self.id), None::<&()>)
            .await?;
        Ok(())
    }

    pub async fn checkout(&self, options: &MachineCheckoutOpts) -> Result<MachineFile, Error> {
        let client = Client::default();
        let mut query = json!({
            "encrypt": 1,
            "include": "license.entitlements"
        });

        if let Some(ttl) = options.ttl {
            query["ttl"] = ttl.into();
        }

        if let Some(ref include) = options.include {
            query["include"] = json!(include.join(","));
        }

        let response = client
            .post(
                &format!("machines/{}/actions/check-out", self.id),
                None::<&()>,
                Some(&query),
            )
            .await?;

        let machine_file_response: CartificateFileResponse = serde_json::from_value(response.body)?;
        let machine_file = MachineFile::from(machine_file_response.data);
        Ok(machine_file)
    }

    pub async fn ping(&self) -> Result<Machine, Error> {
        let client: Client = Client::default();
        let response: Response<MachineResponse> = client
            .post(
                &format!("machines/{}/actions/ping", self.id),
                None::<&()>,
                None::<&()>,
            )
            .await?;
        let machine = Machine::from(response.body.data);
        Ok(machine)
    }

    pub fn monitor(
        self: Arc<Self>,
        heartbeat_interval: Duration,
        tx: Option<Sender<Result<Machine, Error>>>,
        cancel_rx: Option<Receiver<()>>,
    ) -> BoxFuture<'static, ()> {
        async move {
            let send = |result: Result<Machine, Error>| {
                if let Some(tx) = &tx {
                    tx.send(result).unwrap();
                }
            };

            let mut interval_stream = futures::stream::unfold((), move |_| {
                let delay = futures_timer::Delay::new(heartbeat_interval);
                Box::pin(async move {
                    delay.await;
                    Some(((), ()))
                })
            });

            send(self.ping().await);
            while interval_stream.next().await.is_some() {
                match cancel_rx {
                    Some(ref rx) => {
                        if rx.try_recv().is_ok() {
                            break;
                        }
                    }
                    None => {}
                }
                send(self.ping().await);
            }
        }
        .boxed()
    }

    /// Create a new machine
    #[cfg(feature = "token")]
    pub async fn create(request: MachineCreateRequest) -> Result<Machine, Error> {
        let client = Client::default();

        let mut attributes = serde_json::Map::new();
        attributes.insert("fingerprint".to_string(), json!(request.fingerprint));

        if let Some(name) = request.name {
            attributes.insert("name".to_string(), json!(name));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), json!(platform));
        }
        if let Some(hostname) = request.hostname {
            attributes.insert("hostname".to_string(), json!(hostname));
        }
        if let Some(ip) = request.ip {
            attributes.insert("ip".to_string(), json!(ip));
        }
        if let Some(cores) = request.cores {
            attributes.insert("cores".to_string(), json!(cores));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), json!(metadata));
        }

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
        let client = Client::default();

        let mut query_params = Vec::new();
        if let Some(filters) = filters {
            if let Some(license) = filters.license {
                query_params.push(("license".to_string(), license));
            }
            if let Some(user) = filters.user {
                query_params.push(("user".to_string(), user));
            }
            if let Some(platform) = filters.platform {
                query_params.push(("platform".to_string(), platform));
            }
            if let Some(name) = filters.name {
                query_params.push(("name".to_string(), name));
            }
            if let Some(fingerprint) = filters.fingerprint {
                query_params.push(("fingerprint".to_string(), fingerprint));
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
        let machines_response: MachinesResponse = serde_json::from_value(response.body)?;
        Ok(machines_response
            .data
            .into_iter()
            .map(Machine::from)
            .collect())
    }

    /// Get a machine by ID
    #[cfg(feature = "token")]
    pub async fn get(id: &str) -> Result<Machine, Error> {
        let client = Client::default();
        let endpoint = format!("machines/{}", id);
        let response = client.get(&endpoint, None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }

    /// Update a machine
    #[cfg(feature = "token")]
    pub async fn update(&self, request: MachineUpdateRequest) -> Result<Machine, Error> {
        let client = Client::default();
        let endpoint = format!("machines/{}", self.id);

        let mut attributes = serde_json::Map::new();
        if let Some(name) = request.name {
            attributes.insert("name".to_string(), json!(name));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), json!(platform));
        }
        if let Some(hostname) = request.hostname {
            attributes.insert("hostname".to_string(), json!(hostname));
        }
        if let Some(ip) = request.ip {
            attributes.insert("ip".to_string(), json!(ip));
        }
        if let Some(cores) = request.cores {
            attributes.insert("cores".to_string(), json!(cores));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), json!(metadata));
        }

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
        let client = Client::default();
        let endpoint = format!("machines/{}/actions/reset", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let machine_response: MachineResponse = serde_json::from_value(response.body)?;
        Ok(Machine::from(machine_response.data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData};
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
                other: HashMap::new(),
            },
        };

        let machine = Machine::from(machine_data);
        
        assert_eq!(machine.account_id, Some("test-account-id".to_string()));
        assert_eq!(machine.environment_id, Some("test-environment-id".to_string()));
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
}
