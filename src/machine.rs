use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MachineAttributes {
    pub fingerprint: String,
    pub name: Option<String>,
    pub platform: Option<String>,
    pub hostname: Option<String>,
    pub cores: Option<i32>,
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
    pub cores: Option<i32>,
    #[serde(rename = "requireHeartbeat")]
    pub require_heartbeat: bool,
    #[serde(rename = "heartbeatStatus")]
    pub heartbeat_status: String,
    #[serde(rename = "heartbeatDuration")]
    pub heartbeat_duration: Option<i32>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Machine {
    pub(crate) fn from(data: KeygenResponseData<MachineAttributes>) -> Machine {
        Machine {
          id: data.id,
          fingerprint: data.attributes.fingerprint,
          name: data.attributes.name,
          platform: data.attributes.platform,
          hostname: data.attributes.hostname,
          cores: data.attributes.cores,
          require_heartbeat: data.attributes.require_heartbeat,
          heartbeat_status: data.attributes.heartbeat_status,
          heartbeat_duration: data.attributes.heartbeat_duration,
          created: data.attributes.created,
          updated: data.attributes.updated,
        }
    }

    pub async fn deactivate(&self) -> Result<(), Error> {
        let client = Client::default();
        let _response = client
            .delete::<(), serde_json::Value>(&format!("machines/{}", self.id), None::<&()>)
            .await?;
        Ok(())
    }
}
