use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub id: String,
    pub fingerprint: String,
    pub name: String,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub environment_id: Option<String>,
    pub product_id: Option<String>,
    pub license_id: Option<String>,
    pub machine_id: Option<String>,
}

impl From<keygen_rs::component::Component> for Component {
    fn from(c: keygen_rs::component::Component) -> Self {
        Component {
            id: c.id,
            fingerprint: c.fingerprint,
            name: c.name,
            metadata: c
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: c.created.to_rfc3339(),
            updated: c.updated.to_rfc3339(),
            account_id: c.account_id,
            environment_id: c.environment_id,
            product_id: c.product_id,
            license_id: c.license_id,
            machine_id: c.machine_id,
        }
    }
}

#[wasm_bindgen(js_name = "createComponent")]
pub async fn create_component(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        fingerprint: String,
        name: String,
        metadata: Option<serde_json::Value>,
        machine_id: String,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::component::CreateComponentRequest {
        fingerprint: req.fingerprint,
        name: req.name,
        metadata: crate::opt_metadata(req.metadata)?,
        machine_id: req.machine_id,
    };

    let component = keygen_rs::component::Component::create(r)
        .await
        .map(Component::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&component).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listComponents")]
pub async fn list_components(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
        machine: Option<String>,
        license: Option<String>,
        owner: Option<String>,
        user: Option<String>,
        product: Option<String>,
    }
    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let list_opts = opts.map(|o| keygen_rs::component::ListComponentsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        machine: o.machine,
        license: o.license,
        owner: o.owner,
        user: o.user,
        product: o.product,
    });

    let components: Vec<Component> = keygen_rs::component::Component::list(list_opts)
        .await
        .map(|list| list.into_iter().map(Component::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&components).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getComponent")]
pub async fn get_component(id: String) -> Result<JsValue, JsError> {
    let component = keygen_rs::component::Component::get(&id)
        .await
        .map(Component::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&component).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateComponent")]
pub async fn update_component(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;
    let comp = keygen_rs::component::Component {
        id,
        ..Default::default()
    };

    let r = keygen_rs::component::UpdateComponentRequest {
        name: req.name,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let component = comp
        .update(r)
        .await
        .map(Component::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&component).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteComponent")]
pub async fn delete_component(id: String) -> Result<(), JsError> {
    let comp = keygen_rs::component::Component {
        id,
        ..Default::default()
    };
    comp.delete().await.map_err(to_js_error)
}
