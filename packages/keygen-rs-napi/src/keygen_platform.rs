use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Platform {
    pub id: String,
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::platform::Platform> for Platform {
    fn from(p: keygen_rs::platform::Platform) -> Self {
        Platform {
            id: p.id,
            name: p.name,
            key: p.key,
            created: p.created,
            updated: p.updated,
            account_id: p.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct ListPlatformsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi]
pub async fn list_platforms(options: Option<ListPlatformsOptions>) -> Result<Vec<Platform>> {
    let opts = options.map(|o| keygen_rs::platform::ListPlatformsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });
    keygen_rs::platform::Platform::list(opts)
        .await
        .map(|list| list.into_iter().map(Platform::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_platform(id: String) -> Result<Platform> {
    keygen_rs::platform::Platform::get(&id)
        .await
        .map(Platform::from)
        .map_err(to_napi_error)
}
