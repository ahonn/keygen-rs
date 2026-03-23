use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

fn make_entitlement(id: String) -> keygen_rs::entitlement::Entitlement {
    keygen_rs::entitlement::Entitlement {
        id,
        name: None,
        code: String::new(),
        metadata: None,
        created: chrono::Utc::now(),
        updated: chrono::Utc::now(),
        account_id: None,
    }
}

#[wasm_bindgen(js_name = "createEntitlement")]
pub async fn create_entitlement(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        code: String,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::entitlement::CreateEntitlementRequest {
        name: req.name,
        code: req.code,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let entitlement = keygen_rs::entitlement::Entitlement::create(r)
        .await
        .map(Entitlement::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&entitlement).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listEntitlements")]
pub async fn list_entitlements(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let list_opts = opts.map(|o| keygen_rs::entitlement::ListEntitlementsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    let entitlements: Vec<Entitlement> = keygen_rs::entitlement::Entitlement::list(list_opts)
        .await
        .map(|list| list.into_iter().map(Entitlement::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&entitlements).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getEntitlement")]
pub async fn get_entitlement(id: String) -> Result<JsValue, JsError> {
    let entitlement = keygen_rs::entitlement::Entitlement::get(&id)
        .await
        .map(Entitlement::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&entitlement).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateEntitlement")]
pub async fn update_entitlement(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        code: Option<String>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;
    let ent = make_entitlement(id);

    let r = keygen_rs::entitlement::UpdateEntitlementRequest {
        name: req.name,
        code: req.code,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let entitlement = ent
        .update(r)
        .await
        .map(Entitlement::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&entitlement).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteEntitlement")]
pub async fn delete_entitlement(id: String) -> Result<(), JsError> {
    let ent = make_entitlement(id);
    ent.delete().await.map_err(to_js_error)
}
