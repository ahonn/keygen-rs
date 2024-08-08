use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::component::Component;
use crate::errors::Error;
use crate::machine_file::MachineFile;
use crate::process::Process;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: String,
    pub fingerprint: String,
    pub name: String,
    pub hostname: String,
    pub platform: String,
    pub cores: i32,
    pub require_heartbeat: bool,
    pub heartbeat_status: HeartbeatStatus,
    pub heartbeat_duration: i32,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeartbeatStatus {
    NotStarted,
    Alive,
    Dead,
    Resurrected,
}

impl Machine {
    pub async fn deactivate(&self) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn monitor(&self) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn checkout(&self, options: &CheckoutOptions) -> Result<MachineFile, Error> {
        unimplemented!()
    }

    pub async fn components(&self) -> Result<Vec<Component>, Error> {
        unimplemented!()
    }

    pub async fn spawn(&self, pid: &str) -> Result<Process, Error> {
        unimplemented!()
    }

    pub async fn processes(&self) -> Result<Vec<Process>, Error> {
        unimplemented!()
    }
}

pub struct CheckoutOptions {
    // Define checkout options here
}
