use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub filename: String,
    pub filetype: String,
    pub filesize: i64,
    pub platform: String,
    pub arch: String,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub release_id: String,
    pub url: Option<String>,
}
