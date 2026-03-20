use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Arch {
    pub id: String,
    pub name: Option<String>,
    pub key: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::arch::Arch> for Arch {
    fn from(a: keygen_rs::arch::Arch) -> Self {
        Arch {
            id: a.id,
            name: a.name,
            key: a.key,
            created: a.created,
            updated: a.updated,
            account_id: a.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct ListArchesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi]
pub async fn list_arches(options: Option<ListArchesOptions>) -> Result<Vec<Arch>> {
    let opts = options.map(|o| keygen_rs::arch::ListArchesOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });
    keygen_rs::arch::Arch::list(opts)
        .await
        .map(|list| list.into_iter().map(Arch::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_arch(id: String) -> Result<Arch> {
    keygen_rs::arch::Arch::get(&id)
        .await
        .map(Arch::from)
        .map_err(to_napi_error)
}
