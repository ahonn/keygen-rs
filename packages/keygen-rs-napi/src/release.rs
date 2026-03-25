use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::to_napi_error;

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct ReleaseUpgradeRequest {
    pub product: Option<String>,
    pub constraint: Option<String>,
    pub package: Option<String>,
    pub channel: Option<String>,
}

#[napi(object)]
#[derive(Clone)]
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

#[napi(object)]
#[derive(Clone)]
pub struct CreateReleaseRequest {
    pub product_id: String,
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub tag: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct UpdateReleaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub channel: Option<String>,
    pub tag: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ListReleasesOptions {
    pub limit: Option<u32>,
    pub page_size: Option<u32>,
    pub page_number: Option<u32>,
    pub channel: Option<String>,
    pub status: Option<String>,
    pub product: Option<String>,
    pub package: Option<String>,
    pub engine: Option<String>,
    pub entitlements: Option<Vec<String>>,
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
        name: request.name,
        description: request.description,
        status: request
            .status
            .as_deref()
            .map(|s| crate::parse_enum(s, "release status"))
            .transpose()?,
        tag: request.tag,
        metadata: crate::opt_metadata(request.metadata)?,
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
                status: o
                    .status
                    .as_deref()
                    .map(|s| crate::parse_enum(s, "release status"))
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
        package_id: None,
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
        name: request.name,
        description: request.description,
        channel,
        tag: request.tag,
        metadata: crate::opt_metadata(request.metadata)?,
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

#[napi]
pub async fn upgrade_release(
    id: String,
    request: Option<ReleaseUpgradeRequest>,
) -> Result<Release> {
    let rel = make_minimal_release(id);
    let req = request
        .map(
            |request| -> Result<keygen_rs::release::ReleaseUpgradeRequest> {
                Ok(keygen_rs::release::ReleaseUpgradeRequest {
                    product: request.product,
                    constraint: request.constraint,
                    package: request.package,
                    channel: request.channel.as_deref().map(parse_channel).transpose()?,
                })
            },
        )
        .transpose()?;

    rel.upgrade(req.as_ref())
        .await
        .map(Release::from)
        .map_err(to_napi_error)
}

#[napi]
pub async fn download_release_artifact(
    id: String,
    artifact: String,
) -> Result<ReleaseArtifactDownload> {
    let rel = make_minimal_release(id);
    rel.download_artifact(&artifact)
        .await
        .map(|download| ReleaseArtifactDownload {
            location: download.location,
        })
        .map_err(to_napi_error)
}

#[napi]
pub async fn change_release_package(id: String, package_id: String) -> Result<Release> {
    let rel = make_minimal_release(id);
    rel.change_package(&package_id)
        .await
        .map(Release::from)
        .map_err(to_napi_error)
}

#[napi(object)]
#[derive(Clone)]
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

#[napi]
pub async fn release_artifacts(
    id: String,
    options: Option<crate::artifact::ListArtifactsOptions>,
) -> Result<Vec<crate::artifact::Artifact>> {
    let rel = make_minimal_release(id);
    let opts = options.map(|o| keygen_rs::artifact::ListArtifactsOptions {
        limit: o.limit,
        page_size: o.page_size,
        page_number: o.page_number,
        release: o.release,
        product: o.product,
        channel: o.channel,
        platform: o.platform,
        arch: o.arch,
        filetype: o.filetype,
        status: o.status.and_then(|s| {
            serde_json::from_value::<keygen_rs::artifact::ArtifactStatus>(
                serde_json::Value::String(s),
            )
            .ok()
        }),
    });
    rel.artifacts(opts)
        .await
        .map(|artifacts| {
            artifacts
                .into_iter()
                .map(crate::artifact::Artifact::from)
                .collect()
        })
        .map_err(to_napi_error)
}

#[napi]
pub async fn attach_release_constraints(
    id: String,
    entitlement_ids: Vec<String>,
) -> Result<Vec<Constraint>> {
    let rel = make_minimal_release(id);
    rel.attach_constraints(&entitlement_ids)
        .await
        .map(|constraints| constraints.into_iter().map(Constraint::from).collect())
        .map_err(to_napi_error)
}

#[napi]
pub async fn detach_release_constraints(id: String, constraint_ids: Vec<String>) -> Result<()> {
    let rel = make_minimal_release(id);
    rel.detach_constraints(&constraint_ids)
        .await
        .map_err(to_napi_error)
}

#[napi]
pub async fn release_constraints(id: String) -> Result<Vec<Constraint>> {
    let rel = make_minimal_release(id);
    rel.constraints(None)
        .await
        .map(|constraints| constraints.into_iter().map(Constraint::from).collect())
        .map_err(to_napi_error)
}
