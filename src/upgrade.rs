use serde::{Deserialize, Serialize};

use crate::errors::Error;
use crate::release::Release;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    pub current_version: String,
    pub product: Option<String>,
    pub package: Option<String>,
    pub constraint: Option<String>,
    pub channel: Option<String>,
    pub public_key: Option<String>,
    pub filename: Option<String>,
}

pub async fn upgrade(options: Options) -> Result<Release, Error> {
    unimplemented!()
}
