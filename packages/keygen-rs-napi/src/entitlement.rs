use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Entitlement {
    pub id: String,
    pub name: Option<String>,
    pub code: String,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
}

impl From<keygen_rs::entitlement::Entitlement> for Entitlement {
    fn from(e: keygen_rs::entitlement::Entitlement) -> Self {
        Entitlement {
            id: e.id,
            name: e.name,
            code: e.code,
            metadata: e
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: e.created.to_rfc3339(),
            updated: e.updated.to_rfc3339(),
            account_id: e.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateEntitlementRequest {
    pub name: Option<String>,
    pub code: String,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateEntitlementRequest {
    pub name: Option<String>,
    pub code: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListEntitlementsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
}

#[napi]
pub async fn create_entitlement(request: CreateEntitlementRequest) -> Result<Entitlement> {
    let req = keygen_rs::entitlement::CreateEntitlementRequest {
        name: request.name,
        code: request.code,
        metadata: crate::opt_metadata(request.metadata)?,
    };
    keygen_rs::entitlement::Entitlement::create(req)
        .await
        .map(Entitlement::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_entitlements(
    options: Option<ListEntitlementsOptions>,
) -> Result<Vec<Entitlement>> {
    let opts = options.map(|o| keygen_rs::entitlement::ListEntitlementsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });
    keygen_rs::entitlement::Entitlement::list(opts)
        .await
        .map(|list| list.into_iter().map(Entitlement::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_entitlement(id: String) -> Result<Entitlement> {
    keygen_rs::entitlement::Entitlement::get(&id)
        .await
        .map(Entitlement::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_entitlement(
    id: String,
    request: UpdateEntitlementRequest,
) -> Result<Entitlement> {
    let ent = keygen_rs::entitlement::Entitlement {
        id,
        name: None,
        code: String::new(),
        metadata: None,
        created: chrono::Utc::now(),
        updated: chrono::Utc::now(),
        account_id: None,
    };
    let req = keygen_rs::entitlement::UpdateEntitlementRequest {
        name: request.name,
        code: request.code,
        metadata: crate::opt_metadata(request.metadata)?,
    };
    ent.update(req)
        .await
        .map(Entitlement::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_entitlement(id: String) -> Result<()> {
    let ent = keygen_rs::entitlement::Entitlement {
        id,
        name: None,
        code: String::new(),
        metadata: None,
        created: chrono::Utc::now(),
        updated: chrono::Utc::now(),
        account_id: None,
    };
    ent.delete().await.map_err(to_napi_error)
}
