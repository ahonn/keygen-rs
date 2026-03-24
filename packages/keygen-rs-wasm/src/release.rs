use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::to_js_error;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Release {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub channel: Option<String>,
    pub status: String,
    pub tag: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub product_id: Option<String>,
    pub package_id: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseArtifactDownload {
    pub location: String,
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
            name: r.name,
            description: r.description,
            version: r.version,
            channel,
            status,
            tag: r.tag,
            metadata: r
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: r.created,
            updated: r.updated,
            yanked_at: r.yanked_at,
            product_id: r.product_id,
            package_id: r.package_id,
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
        package_id: None,
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
        name: Option<String>,
        description: Option<String>,
        status: Option<String>,
        tag: Option<String>,
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
        name: req.name,
        description: req.description,
        status: req
            .status
            .as_deref()
            .map(|s| {
                serde_json::from_value::<keygen_rs::release::ReleaseStatus>(
                    serde_json::Value::String(s.to_string()),
                )
                .map_err(|e| JsError::new(&format!("Invalid release status: {e}")))
            })
            .transpose()?,
        tag: req.tag,
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
        status: Option<String>,
        product: Option<String>,
        package: Option<String>,
        engine: Option<String>,
        entitlements: Option<Vec<String>>,
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
                status: o
                    .status
                    .as_deref()
                    .map(|s| {
                        crate::parse_enum::<keygen_rs::release::ReleaseStatus>(s, "release status")
                    })
                    .transpose()?,
                version: None,
                product: o.product,
                package: o.package,
                engine: o.engine,
                entitlements: o.entitlements,
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

#[wasm_bindgen(js_name = "upgradeRelease")]
pub async fn upgrade_release(id: String, request: JsValue) -> Result<JsValue, JsError> {
    #[derive(Deserialize, Default)]
    #[serde(rename_all = "camelCase")]
    struct Req {
        product: Option<String>,
        constraint: Option<String>,
        package: Option<String>,
        channel: Option<String>,
    }

    let req: Option<Req> = if request.is_undefined() || request.is_null() {
        None
    } else {
        Some(serde_wasm_bindgen::from_value(request).map_err(|e| JsError::new(&e.to_string()))?)
    };

    let req = req
        .map(
            |request| -> Result<keygen_rs::release::ReleaseUpgradeRequest, JsError> {
                Ok(keygen_rs::release::ReleaseUpgradeRequest {
                    product: request.product,
                    constraint: request.constraint,
                    package: request.package,
                    channel: request.channel.as_deref().map(parse_channel).transpose()?,
                })
            },
        )
        .transpose()?;

    let release = make_minimal_release(id)
        .upgrade(req.as_ref())
        .await
        .map(Release::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "downloadReleaseArtifact")]
pub async fn download_release_artifact(id: String, artifact: String) -> Result<JsValue, JsError> {
    let download = make_minimal_release(id)
        .download_artifact(&artifact)
        .await
        .map(|download| ReleaseArtifactDownload {
            location: download.location,
        })
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&download).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "changeReleasePackage")]
pub async fn change_release_package(id: String, package_id: String) -> Result<JsValue, JsError> {
    let release = make_minimal_release(id)
        .change_package(&package_id)
        .await
        .map(Release::from)
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&release).map_err(|e| JsError::new(&e.to_string()))
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Constraint {
    pub id: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub entitlement_id: Option<String>,
    pub release_id: Option<String>,
}

impl From<keygen_rs::release::Constraint> for Constraint {
    fn from(c: keygen_rs::release::Constraint) -> Self {
        Constraint {
            id: c.id,
            created: c.created,
            updated: c.updated,
            account_id: c.account_id,
            entitlement_id: c.entitlement_id,
            release_id: c.release_id,
        }
    }
}

#[wasm_bindgen(js_name = "releaseArtifacts")]
pub async fn release_artifacts(id: String, options: JsValue) -> Result<JsValue, JsError> {
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
    let list_opts = opts.map(|o| keygen_rs::artifact::ListArtifactsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        ..Default::default()
    });

    #[derive(Clone, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct WasmArtifact {
        id: String,
        filename: String,
        filetype: Option<String>,
        filesize: Option<u64>,
        platform: Option<String>,
        arch: Option<String>,
        signature: Option<String>,
        checksum: Option<String>,
        status: String,
        created: String,
        updated: String,
    }

    let rel = make_minimal_release(id);
    let artifacts: Vec<WasmArtifact> = rel
        .artifacts(list_opts)
        .await
        .map(|artifacts| {
            artifacts
                .into_iter()
                .map(|a| {
                    let status = serde_json::to_value(&a.status)
                        .ok()
                        .and_then(|v| v.as_str().map(String::from))
                        .unwrap_or_default();
                    WasmArtifact {
                        id: a.id,
                        filename: a.filename,
                        filetype: a.filetype,
                        filesize: a.filesize,
                        platform: a.platform,
                        arch: a.arch,
                        signature: a.signature,
                        checksum: a.checksum,
                        status,
                        created: a.created,
                        updated: a.updated,
                    }
                })
                .collect()
        })
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&artifacts).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "attachReleaseConstraints")]
pub async fn attach_release_constraints(
    id: String,
    entitlement_ids: Vec<String>,
) -> Result<JsValue, JsError> {
    let rel = make_minimal_release(id);
    let constraints: Vec<Constraint> = rel
        .attach_constraints(&entitlement_ids)
        .await
        .map(|constraints| constraints.into_iter().map(Constraint::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&constraints).map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen(js_name = "detachReleaseConstraints")]
pub async fn detach_release_constraints(
    id: String,
    constraint_ids: Vec<String>,
) -> Result<(), JsError> {
    let rel = make_minimal_release(id);
    rel.detach_constraints(&constraint_ids)
        .await
        .map_err(to_js_error)
}

#[wasm_bindgen(js_name = "releaseConstraints")]
pub async fn release_constraints(id: String) -> Result<JsValue, JsError> {
    let rel = make_minimal_release(id);
    let constraints: Vec<Constraint> = rel
        .constraints(None)
        .await
        .map(|constraints| constraints.into_iter().map(Constraint::from).collect())
        .map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&constraints).map_err(|e| JsError::new(&e.to_string()))
}
