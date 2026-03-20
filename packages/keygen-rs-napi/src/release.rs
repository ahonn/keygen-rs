use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct CreateReleaseRequest {
    pub product_id: String,
    pub version: String,
    pub channel: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateReleaseRequest {
    pub version: Option<String>,
    pub channel: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListReleasesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub channel: Option<String>,
    pub product: Option<String>,
}

fn to_hashmap(
    val: Option<serde_json::Value>,
) -> Option<std::collections::HashMap<String, serde_json::Value>> {
    val.and_then(|v| serde_json::from_value(v).ok())
}

fn parse_channel(s: &str) -> std::result::Result<keygen_rs::release::ReleaseChannel, napi::Error> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| napi::Error::new(Status::InvalidArg, format!("Invalid release channel: {e}")))
}

#[napi]
pub async fn create_release(request: CreateReleaseRequest) -> Result<Release> {
    let channel = match &request.channel {
        Some(c) => parse_channel(c)?,
        None => keygen_rs::release::ReleaseChannel::Stable,
    };
    let req = keygen_rs::release::CreateReleaseRequest {
        version: request.version,
        channel,
        product_id: request.product_id,
        name: None,
        description: None,
        status: None,
        tag: None,
        metadata: to_hashmap(request.metadata),
    };
    keygen_rs::release::Release::create(req)
        .await
        .map(Release::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn list_releases(options: Option<ListReleasesOptions>) -> Result<Vec<Release>> {
    let opts = match options {
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
    keygen_rs::release::Release::list(opts)
        .await
        .map(|list| list.into_iter().map(Release::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn get_release(id: String) -> Result<Release> {
    keygen_rs::release::Release::get(&id)
        .await
        .map(Release::from)
        .map_err(to_napi_error)
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

#[napi]
pub async fn update_release(id: String, request: UpdateReleaseRequest) -> Result<Release> {
    let rel = make_minimal_release(id);
    let channel = match request.channel {
        Some(ref c) => Some(parse_channel(c)?),
        None => None,
    };
    let req = keygen_rs::release::UpdateReleaseRequest {
        name: None,
        description: None,
        channel,
        tag: None,
        metadata: to_hashmap(request.metadata),
    };
    rel.update(req)
        .await
        .map(Release::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn delete_release(id: String) -> Result<()> {
    let rel = make_minimal_release(id);
    rel.delete().await.map_err(to_napi_error)
}

#[napi]
pub async fn publish_release(id: String) -> Result<Release> {
    let rel = make_minimal_release(id);
    rel.publish()
        .await
        .map(Release::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn yank_release(id: String) -> Result<Release> {
    let rel = make_minimal_release(id);
    rel.yank().await.map(Release::from).map_err(to_napi_error)
}
