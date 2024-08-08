use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub enc: String,
    pub sig: String,
    pub alg: String,
}
