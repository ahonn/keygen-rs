use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::KeygenResponseData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitlementAttributes {
    pub name: Option<String>,
    pub code: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct EntitlementsResponse {
    pub data: Vec<KeygenResponseData<EntitlementAttributes>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entitlement {
    pub id: String,
    pub attributes: EntitlementAttributes,
}
