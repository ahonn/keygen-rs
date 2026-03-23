use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub max_users: Option<i32>,
    pub max_licenses: Option<i32>,
    pub max_machines: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub owner_id: Option<String>,
}

impl From<keygen_rs::group::Group> for Group {
    fn from(g: keygen_rs::group::Group) -> Self {
        Group {
            id: g.id,
            name: g.name,
            max_users: g.max_users,
            max_licenses: g.max_licenses,
            max_machines: g.max_machines,
            metadata: g
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: g.created.to_rfc3339(),
            updated: g.updated.to_rfc3339(),
            account_id: g.account_id,
            owner_id: g.owner_id,
        }
    }
}

#[wasm_bindgen(js_name = "createGroup")]
pub async fn create_group(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: String,
        max_users: Option<i32>,
        max_licenses: Option<i32>,
        max_machines: Option<i32>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let r = keygen_rs::group::CreateGroupRequest {
        name: req.name,
        max_users: req.max_users,
        max_licenses: req.max_licenses,
        max_machines: req.max_machines,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let group = keygen_rs::group::Group::create(r)
        .await
        .map(Group::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&group).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listGroups")]
pub async fn list_groups(options: JsValue) -> Result<JsValue, JsError> {
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

    let list_opts = opts.map(|o| keygen_rs::group::ListGroupsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
    });

    let groups: Vec<Group> = keygen_rs::group::Group::list(list_opts)
        .await
        .map(|list| list.into_iter().map(Group::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&groups).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getGroup")]
pub async fn get_group(id: String) -> Result<JsValue, JsError> {
    let group = keygen_rs::group::Group::get(&id)
        .await
        .map(Group::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&group).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateGroup")]
pub async fn update_group(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        max_users: Option<i32>,
        max_licenses: Option<i32>,
        max_machines: Option<i32>,
        metadata: Option<serde_json::Value>,
    }
    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let grp = keygen_rs::group::Group {
        id,
        ..Default::default()
    };

    let r = keygen_rs::group::UpdateGroupRequest {
        name: req.name,
        max_users: req.max_users,
        max_licenses: req.max_licenses,
        max_machines: req.max_machines,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let group = grp.update(r).await.map(Group::from).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&group).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteGroup")]
pub async fn delete_group(id: String) -> Result<(), JsError> {
    let grp = keygen_rs::group::Group {
        id,
        ..Default::default()
    };
    grp.delete().await.map_err(to_js_error)
}
