use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub id: String,
    pub pid: String,
    pub status: ProcessStatus,
    pub interval: i32,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub machine_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessStatus {
    Alive,
    Dead,
}

impl Process {
    pub async fn kill(&self) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn monitor(&self) -> Result<(), Error> {
        unimplemented!()
    }
}
