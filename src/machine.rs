use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::Client;
use crate::errors::Error;
use crate::KeygenResponseData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineAttributes {
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
pub struct MachineResponse {
    pub data: KeygenResponseData<MachineAttributes>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachinesResponse {
    pub data: Vec<KeygenResponseData<MachineAttributes>>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub attributes: MachineAttributes,
}

impl Machine {
    pub async fn deactivate(&self) -> Result<(), Error> {
        let client = Client::default();
        let _response = client
            .delete::<(), serde_json::Value>(&format!("machines/{}", self.id), None::<&()>)
            .await?;
        Ok(())
    }
}

pub struct CheckoutOptions {
    // Define checkout options here
}
