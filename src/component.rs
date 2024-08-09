use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub fingerprint: String,
    pub name: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
}

impl Component {
    pub fn create_object(component: &Component) -> serde_json::Value {
        json!({
          "data": {
            "id": component.id,
            "type": "components",
            "attributes": component
          }
        })
    }
}
