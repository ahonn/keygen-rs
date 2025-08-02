use serde::{Deserialize, Serialize};

/// Simple Group representation for included relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}