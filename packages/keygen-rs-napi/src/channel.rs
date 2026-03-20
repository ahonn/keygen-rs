use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Channel {
    pub id: String,
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::channel::Channel> for Channel {
    fn from(c: keygen_rs::channel::Channel) -> Self {
        Channel {
            id: c.id,
            name: c.name,
            key: c.key,
            created: c.created,
            updated: c.updated,
            account_id: c.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct ListChannelsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi]
pub async fn list_channels(options: Option<ListChannelsOptions>) -> Result<Vec<Channel>> {
    let opts = options.map(|o| keygen_rs::channel::ListChannelsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });
    keygen_rs::channel::Channel::list(opts)
        .await
        .map(|list| list.into_iter().map(Channel::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_channel(id: String) -> Result<Channel> {
    keygen_rs::channel::Channel::get(&id)
        .await
        .map(Channel::from)
        .map_err(to_napi_error)
}
