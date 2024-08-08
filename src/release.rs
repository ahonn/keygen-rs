use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub channel: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Release {
    pub async fn install(&self) -> Result<(), Error> {
        unimplemented!()
    }
}
