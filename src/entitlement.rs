use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub id: String,
    pub code: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}
