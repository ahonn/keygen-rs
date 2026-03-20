use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
pub struct Artifact {
    pub id: String,
    pub filename: String,
    pub filetype: String,
    pub filesize: i64,
    pub platform: String,
    pub arch: String,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub release_id: Option<String>,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
}

impl From<keygen_rs::artifact::Artifact> for Artifact {
    fn from(a: keygen_rs::artifact::Artifact) -> Self {
        let status = serde_json::to_value(&a.status)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        Artifact {
            id: a.id,
            filename: a.filename,
            filetype: a.filetype.unwrap_or_default(),
            filesize: a.filesize.map(|s| s as i64).unwrap_or(0),
            platform: a.platform.unwrap_or_default(),
            arch: a.arch.unwrap_or_default(),
            signature: a.signature,
            checksum: a.checksum,
            status,
            metadata: a
                .metadata
                .map(|m| serde_json::to_value(m).unwrap_or_default()),
            created: a.created,
            updated: a.updated,
            yanked_at: a.yanked_at,
            release_id: a.release_id,
            product_id: a.product_id,
            account_id: a.account_id,
        }
    }
}

#[napi(object)]
#[derive(Clone)]
pub struct CreateArtifactRequest {
    pub release_id: String,
    pub filename: String,
    pub filetype: String,
    pub filesize: i64,
    pub platform: String,
    pub arch: String,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateArtifactRequest {
    pub filename: Option<String>,
    pub filetype: Option<String>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListArtifactsOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub release: Option<String>,
    pub product: Option<String>,
    pub channel: Option<String>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub filetype: Option<String>,
    pub status: Option<String>,
}

fn to_hashmap(
    val: Option<serde_json::Value>,
) -> Option<std::collections::HashMap<String, serde_json::Value>> {
    val.and_then(|v| serde_json::from_value(v).ok())
}

fn make_minimal_artifact(id: String) -> keygen_rs::artifact::Artifact {
    keygen_rs::artifact::Artifact {
        id,
        filename: String::new(),
        filetype: None,
        filesize: None,
        platform: None,
        arch: None,
        signature: None,
        checksum: None,
        status: keygen_rs::artifact::ArtifactStatus::Waiting,
        metadata: None,
        created: String::new(),
        updated: String::new(),
        yanked_at: None,
        release_id: None,
        product_id: None,
        account_id: None,
    }
}

#[napi]
pub async fn create_artifact(request: CreateArtifactRequest) -> Result<Artifact> {
    let req = keygen_rs::artifact::CreateArtifactRequest {
        filename: request.filename,
        release_id: request.release_id,
        filetype: Some(request.filetype),
        filesize: Some(request.filesize as u64),
        platform: Some(request.platform),
        arch: Some(request.arch),
        signature: request.signature,
        checksum: request.checksum,
        metadata: to_hashmap(request.metadata),
    };
    keygen_rs::artifact::Artifact::create(req)
        .await
        .map(Artifact::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_artifacts(options: Option<ListArtifactsOptions>) -> Result<Vec<Artifact>> {
    let opts = match options {
        Some(o) => {
            let status = o.status.and_then(|s| {
                serde_json::from_value::<keygen_rs::artifact::ArtifactStatus>(
                    serde_json::Value::String(s),
                )
                .ok()
            });
            Some(keygen_rs::artifact::ListArtifactsOptions {
                limit: o.limit,
                page_size: o.page_size,
                page_number: o.page_number,
                release: o.release,
                product: o.product,
                channel: o.channel,
                platform: o.platform,
                arch: o.arch,
                filetype: o.filetype,
                status,
            })
        }
        None => None,
    };
    keygen_rs::artifact::Artifact::list(opts)
        .await
        .map(|list| list.into_iter().map(Artifact::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_artifact(id: String) -> Result<Artifact> {
    keygen_rs::artifact::Artifact::get(&id)
        .await
        .map(Artifact::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn update_artifact(id: String, request: UpdateArtifactRequest) -> Result<Artifact> {
    let art = make_minimal_artifact(id);
    let req = keygen_rs::artifact::UpdateArtifactRequest {
        filename: request.filename,
        filetype: request.filetype,
        platform: request.platform,
        arch: request.arch,
        signature: request.signature,
        checksum: request.checksum,
        metadata: to_hashmap(request.metadata),
    };
    art.update(req)
        .await
        .map(Artifact::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_artifact(id: String) -> Result<()> {
    let art = make_minimal_artifact(id);
    art.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn yank_artifact(id: String) -> Result<Artifact> {
    let art = make_minimal_artifact(id);
    art.yank().await.map(Artifact::from).map_err(to_napi_error)
}
