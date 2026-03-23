use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    pub version: String,
    pub channel: Option<String>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
}

impl From<keygen_rs::release::Release> for Release {
    fn from(r: keygen_rs::release::Release) -> Self {
        let channel = serde_json::to_value(&r.channel)
            .ok()
            .and_then(|v| v.as_str().map(String::from));
        let status = serde_json::to_value(&r.status)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        Release {
            id: r.id,
            version: r.version,
            channel,
            status,
            metadata: r
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: r.created,
            updated: r.updated,
            product_id: r.product_id,
            account_id: r.account_id,
        }
    }
}

fn parse_channel(s: &str) -> Result<keygen_rs::release::ReleaseChannel, JsError> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| JsError::new(&format!("Invalid release channel: {e}")))
}

fn make_minimal_release(id: String) -> keygen_rs::release::Release {
    keygen_rs::release::Release {
        id,
        name: None,
        description: None,
        version: String::new(),
        semver: None,
        channel: keygen_rs::release::ReleaseChannel::Stable,
        status: keygen_rs::release::ReleaseStatus::Draft,
        tag: None,
        metadata: None,
        created: String::new(),
        updated: String::new(),
        yanked_at: None,
        product_id: None,
        account_id: None,
    }
}

#[wasm_bindgen(js_name = "createRelease")]
pub async fn create_release(request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        product_id: String,
        version: String,
        channel: Option<String>,
        metadata: Option<serde_json::Value>,
    }

    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let channel = match &req.channel {
        Some(c) => parse_channel(c)?,
        None => keygen_rs::release::ReleaseChannel::Stable,
    };

    let r = keygen_rs::release::CreateReleaseRequest {
        version: req.version,
        channel,
        product_id: req.product_id,
        name: None,
        description: None,
        status: None,
        tag: None,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let release = keygen_rs::release::Release::create(r)
        .await
        .map(Release::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "listReleases")]
pub async fn list_releases(options: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Opts {
        limit: Option<u32>,
        page_size: Option<u32>,
        page_number: Option<u32>,
        channel: Option<String>,
        product: Option<String>,
    }

    let opts: Option<Opts> = if options.is_undefined() || options.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(options).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let keygen_opts = match opts {
        Some(o) => {
            let channel = match o.channel {
                Some(ref c) => Some(parse_channel(c)?),
                None => None,
            };
            Some(keygen_rs::release::ListReleasesOptions {
                limit: o.limit,
                page_size: o.page_size,
                page_number: o.page_number,
                channel,
                status: None,
                version: None,
                product: o.product,
            })
        }
        None => None,
    };

    let releases: Vec<Release> = keygen_rs::release::Release::list(keygen_opts)
        .await
        .map(|list| list.into_iter().map(Release::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&releases).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "getRelease")]
pub async fn get_release(id: String) -> Result<JsValue, JsError> {
    let release = keygen_rs::release::Release::get(&id)
        .await
        .map(Release::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "updateRelease")]
pub async fn update_release(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        name: Option<String>,
        description: Option<String>,
        channel: Option<String>,
        tag: Option<String>,
        metadata: Option<serde_json::Value>,
    }

    let req: Req =
        serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?;

    let channel = match req.channel {
        Some(ref c) => Some(parse_channel(c)?),
        None => None,
    };

    let r = keygen_rs::release::UpdateReleaseRequest {
        name: req.name,
        description: req.description,
        channel,
        tag: req.tag,
        metadata: crate::opt_metadata(req.metadata)?,
    };

    let rel = make_minimal_release(id);
    let release = rel
        .update(r)
        .await
        .map(Release::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "deleteRelease")]
pub async fn delete_release(id: String) -> Result<(), JsError> {
    let rel = make_minimal_release(id);
    rel.delete().await.map_err(to_js_error)
}

#[wasm_bindgen(js_name = "publishRelease")]
pub async fn publish_release(id: String) -> Result<JsValue, JsError> {
    let rel = make_minimal_release(id);
    let release = rel
        .publish()
        .await
        .map(Release::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "yankRelease")]
pub async fn yank_release(id: String) -> Result<JsValue, JsError> {
    let rel = make_minimal_release(id);
    let release = rel.yank().await.map(Release::from).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}
