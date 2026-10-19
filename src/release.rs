#[cfg(not(target_arch = "wasm32"))]
use crate::artifact::{parse_signing_public_key, ArtifactResponse, ArtifactVerifier};
use crate::artifact::{Artifact, ListArtifactsOptions};
use crate::client::{Client, ClientOptions};
use crate::config::{get_config, KeygenConfig};
use crate::errors::{ApiErrorKind, Error};
#[cfg(feature = "token")]
use crate::insert_optional;
use crate::license::PaginationOptions;
use crate::KeygenRelationship;
use crate::KeygenResponseData;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::{header::HeaderMap, redirect::Policy, Client as ReqwestClient};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
#[cfg(not(target_arch = "wasm32"))]
use tokio::io::{AsyncWrite, AsyncWriteExt};

fn serialize_string_vec<S>(value: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(values) => values.serialize(serializer),
        None => serializer.serialize_none(),
    }
}

/// Release channel for distribution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Rc,
    Beta,
    Alpha,
    Dev,
}

/// Release status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseStatus {
    Draft,
    Published,
    Yanked,
}

/// Semantic version components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

/// Release attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAttributes {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub semver: Option<Semver>,
    pub channel: ReleaseChannel,
    pub status: ReleaseStatus,
    pub tag: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    #[serde(rename = "yanked")]
    pub yanked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReleaseResponse {
    pub data: KeygenResponseData<ReleaseAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ReleasesResponse {
    pub data: Vec<KeygenResponseData<ReleaseAttributes>>,
}

/// Request to create a new release
#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReleaseRequest {
    /// Version string (semver format, without 'v' prefix)
    pub version: String,
    /// Release channel
    pub channel: ReleaseChannel,
    /// Associated product ID
    pub product_id: String,
    /// Optional: Human-readable name
    pub name: Option<String>,
    /// Optional: Description or release notes
    pub description: Option<String>,
    /// Optional: Initial status (defaults to DRAFT)
    pub status: Option<ReleaseStatus>,
    /// Optional: Unique tag for lookups
    pub tag: Option<String>,
    /// Optional: Custom metadata (e.g., checksums)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing release
#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateReleaseRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub channel: Option<ReleaseChannel>,
    pub tag: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for listing releases
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListReleasesOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Filter by channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<ReleaseChannel>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ReleaseStatus>,
    /// Filter by version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Filter by product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Filter by package ID or key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Filter by engine ID or key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// Filter by entitlement codes
    #[serde(
        rename = "entitlements[]",
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_string_vec"
    )]
    pub entitlements: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReleaseUpgradeRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<ReleaseChannel>,
}

impl ReleaseUpgradeRequest {
    /// Fill unset `product`/`package` from the config and `channel` with stable.
    fn with_defaults(request: Option<&Self>, config: &KeygenConfig) -> Self {
        let request = request.cloned().unwrap_or_default();
        let non_empty = |value: &str| (!value.is_empty()).then(|| value.to_string());
        Self {
            product: request.product.or_else(|| non_empty(&config.product)),
            package: request.package.or_else(|| non_empty(&config.package)),
            constraint: request.constraint,
            channel: Some(request.channel.unwrap_or(ReleaseChannel::Stable)),
        }
    }
}

pub(crate) async fn upgrade_from_with_client(
    client: &Client,
    config: &KeygenConfig,
    current_version: &str,
    request: Option<&ReleaseUpgradeRequest>,
) -> Result<Option<Release>, Error> {
    let endpoint = format!("releases/{current_version}/upgrade");
    let request = ReleaseUpgradeRequest::with_defaults(request, config);
    match client.get(&endpoint, Some(&request)).await {
        Ok(response) => {
            let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
            Ok(Some(Release::from(release_response.data)))
        }
        Err(err) if err.api_error_kind() == ApiErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct ReleaseArtifactDownload {
    pub location: String,
    pub headers: HeaderMap,
}

/// Request the artifact download: returns the redirect URL of the file and
/// the artifact (with its checksum and signature) from the response body.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn artifact_redirect_with_client(
    client: &Client,
    release_id: &str,
    artifact: &str,
) -> Result<(String, Artifact), Error> {
    let path = format!("releases/{release_id}/artifacts/{artifact}");
    let response = client.get::<(), serde_json::Value>(&path, None).await?;

    let location = response
        .headers
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| Error::UnexpectedError("Missing redirect Location header".to_string()))?
        .to_string();
    if response.body.is_null() {
        return Err(Error::InvalidResponse(
            "artifact download response has no artifact".to_string(),
        ));
    }
    let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;

    Ok((location, Artifact::from(artifact_response.data)))
}

/// Download progress callback: downloaded bytes and the file size, if known.
#[cfg(not(target_arch = "wasm32"))]
pub type DownloadProgress = Box<dyn FnMut(u64, Option<u64>) + Send>;

/// Options for [`Release::download_artifact_to`].
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct ArtifactDownloadOptions {
    /// Hex Ed25519 public key from `keygen genkey` (not the account public key).
    /// When set, the artifact must be signed and its checksum is optional;
    /// when unset, only the checksum is verified and it is required.
    pub public_key: Option<String>,
    /// Called with the downloaded byte count and the file size, if known.
    pub on_progress: Option<DownloadProgress>,
}

#[cfg(not(target_arch = "wasm32"))]
impl std::fmt::Debug for ArtifactDownloadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactDownloadOptions")
            .field("public_key", &self.public_key)
            .field("on_progress", &self.on_progress.is_some())
            .finish()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn download_to_writer_with_client<W>(
    client: &Client,
    config: &KeygenConfig,
    release: &Release,
    artifact: &str,
    writer: &mut W,
    options: Option<ArtifactDownloadOptions>,
) -> Result<Artifact, Error>
where
    W: AsyncWrite + Unpin,
{
    let mut options = options.unwrap_or_default();
    if let Some(public_key) = &options.public_key {
        let is_account_key = config
            .public_key
            .as_deref()
            .is_some_and(|account_key| account_key.eq_ignore_ascii_case(public_key));
        if is_account_key {
            return Err(Error::ArtifactPublicKeyInvalid(
                "the account public key does not sign artifacts, use the key from `keygen genkey`"
                    .to_string(),
            ));
        }
        parse_signing_public_key(public_key)?;
    }

    let (location, artifact) = artifact_redirect_with_client(client, &release.id, artifact).await?;

    // Ed25519ph context is the product ID, as in keygen-go and `keygen upload`.
    // The release's product comes first: `upgrade_from` may target another product.
    let context = release
        .product_id
        .as_deref()
        .or(Some(config.product.as_str()).filter(|product| !product.is_empty()));
    let context = match (&options.public_key, context) {
        (Some(_), None) => {
            return Err(Error::ArtifactSignatureInvalid(
                "product ID for the signature context is unknown".to_string(),
            ))
        }
        (_, context) => context.unwrap_or_default(),
    };
    let mut verifier = ArtifactVerifier::new(&artifact, options.public_key.as_deref(), context)?;

    // Presigned storage URL: no Keygen auth, and no overall timeout so that large
    // files are not cut off — only stalled connections are.
    let http = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        .read_timeout(Duration::from_secs(30))
        .build()?;
    let mut response = http.get(&location).send().await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedError(format!(
            "Artifact download failed with status {}",
            response.status()
        )));
    }

    let mut downloaded = 0u64;
    while let Some(chunk) = response.chunk().await? {
        writer
            .write_all(&chunk)
            .await
            .map_err(|e| Error::UnexpectedError(format!("Failed to write artifact: {e}")))?;
        verifier.update(&chunk);
        downloaded += chunk.len() as u64;
        if let Some(on_progress) = options.on_progress.as_mut() {
            on_progress(downloaded, artifact.filesize);
        }
    }
    writer
        .flush()
        .await
        .map_err(|e| Error::UnexpectedError(format!("Failed to write artifact: {e}")))?;

    verifier.finish()?;
    Ok(artifact)
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn download_with_client(
    client: &Client,
    config: &KeygenConfig,
    release: &Release,
    artifact: &str,
    path: &Path,
    options: Option<ArtifactDownloadOptions>,
) -> Result<Artifact, Error> {
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let part = PathBuf::from(part);

    let result = async {
        let io_error =
            |e: std::io::Error| Error::UnexpectedError(format!("Failed to write artifact: {e}"));
        let mut file = tokio::fs::File::create(&part).await.map_err(io_error)?;
        let artifact =
            download_to_writer_with_client(client, config, release, artifact, &mut file, options)
                .await?;
        file.sync_all().await.map_err(io_error)?;
        drop(file);
        tokio::fs::rename(&part, path).await.map_err(io_error)?;
        Ok(artifact)
    }
    .await;

    if result.is_err() {
        let _ = tokio::fs::remove_file(&part).await;
    }
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintAttributes {
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub created: String,
    pub updated: String,
    pub account_id: Option<String>,
    pub entitlement_id: Option<String>,
    pub release_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConstraintsResponse {
    pub data: Vec<KeygenResponseData<ConstraintAttributes>>,
}

/// A release represents a specific version of your software
#[derive(Debug, Clone)]
pub struct Release {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub semver: Option<Semver>,
    pub channel: ReleaseChannel,
    pub status: ReleaseStatus,
    pub tag: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub product_id: Option<String>,
    pub package_id: Option<String>,
    pub account_id: Option<String>,
}

impl Release {
    pub(crate) fn from(data: KeygenResponseData<ReleaseAttributes>) -> Release {
        Release {
            id: data.id,
            name: data.attributes.name,
            description: data.attributes.description,
            version: data.attributes.version,
            semver: data.attributes.semver,
            channel: data.attributes.channel,
            status: data.attributes.status,
            tag: data.attributes.tag,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            yanked_at: data.attributes.yanked_at,
            product_id: data
                .relationships
                .product
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            package_id: data
                .relationships
                .other
                .get("package")
                .and_then(|value| serde_json::from_value::<KeygenRelationship>(value.clone()).ok())
                .and_then(|rel| rel.data.map(|d| d.id)),
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// List releases with optional filtering and pagination
    pub async fn list(options: Option<ListReleasesOptions>) -> Result<Vec<Release>, Error> {
        let client = Client::from_global_config()?;
        let response = client.get("releases", options.as_ref()).await?;
        let releases_response: ReleasesResponse = serde_json::from_value(response.body)?;
        Ok(releases_response
            .data
            .into_iter()
            .map(Release::from)
            .collect())
    }

    /// Get a release by ID
    pub async fn get(id: &str) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Upgrade a release according to the provided constraints.
    pub async fn upgrade(&self, request: Option<&ReleaseUpgradeRequest>) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/upgrade", self.id);
        let response = client.get(&endpoint, request).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Check whether an upgrade is available for `current_version` (e.g., `1.0.0`).
    ///
    /// Unset `product` and `package` default to [`KeygenConfig::product`] and
    /// [`KeygenConfig::package`]. Unset `channel` defaults to [`ReleaseChannel::Stable`].
    ///
    /// Returns `Ok(None)` when Keygen responds with `NOT_FOUND`. Keygen also does that
    /// for unknown versions and inaccessible products, so a misconfigured product
    /// looks like "no upgrade available".
    pub async fn upgrade_from(
        current_version: &str,
        request: Option<&ReleaseUpgradeRequest>,
    ) -> Result<Option<Release>, Error> {
        let config = get_config()?;
        let client = Client::new(ClientOptions::from(config.clone()))?;
        upgrade_from_with_client(&client, &config, current_version, request).await
    }

    /// Build the download URL for an artifact by ID or filename.
    ///
    /// Returns the fully-qualified API URL that can be used to initiate a download.
    /// On WASM targets, use this URL directly from JavaScript with appropriate
    /// auth headers — the server will respond with a redirect to the actual file.
    pub fn artifact_download_url(&self, artifact: &str) -> Result<String, Error> {
        let client = Client::from_global_config()?;
        let path = format!("releases/{}/artifacts/{}", self.id, artifact);
        let url = client.build_url(&path)?;
        Ok(url.to_string())
    }

    /// Download an artifact by ID or filename, returning the redirect URL.
    ///
    /// Not available on WASM targets — use [`artifact_download_url`](Self::artifact_download_url)
    /// instead and handle the redirect from JavaScript.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn download_artifact(
        &self,
        artifact: &str,
    ) -> Result<ReleaseArtifactDownload, Error> {
        let client = Client::from_global_config()?;
        let path = format!("releases/{}/artifacts/{}", self.id, artifact);
        let request = client.build_request(reqwest::Method::GET, &path, None::<&()>, true)?;

        let no_redirect = ReqwestClient::builder()
            .redirect(Policy::none())
            .build()
            .map_err(|e| Error::UnexpectedError(format!("Failed to build HTTP client: {e}")))?;

        let response = no_redirect.execute(request).await?;

        if response.status().is_client_error() || response.status().is_server_error() {
            let body = response.json().await?;
            return Err(Error::KeygenApiError {
                code: "DOWNLOAD_FAILED".to_string(),
                detail: "Failed to download release artifact".to_string(),
                body,
            });
        }

        let headers = response.headers().clone();
        let location = headers
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| Error::UnexpectedError("Missing redirect Location header".to_string()))?
            .to_string();

        Ok(ReleaseArtifactDownload { location, headers })
    }

    /// Download an artifact by ID or filename to `path` and verify it.
    ///
    /// The file is written to `<path>.part` and renamed to `path` only after it was
    /// verified; on error nothing is left behind. With [`ArtifactDownloadOptions::public_key`]
    /// the signature is required (it covers the SHA-512 of the file) and the checksum is
    /// checked if present; without it the checksum is required.
    ///
    /// The signature context is the release's product, falling back to
    /// [`KeygenConfig::product`]. Replacing the running executable is left to the application.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn download_artifact_to(
        &self,
        artifact: &str,
        path: impl AsRef<Path>,
        options: Option<ArtifactDownloadOptions>,
    ) -> Result<Artifact, Error> {
        let config = get_config()?;
        let client = Client::new(ClientOptions::from(config.clone()))?;
        download_with_client(&client, &config, self, artifact, path.as_ref(), options).await
    }

    /// List artifacts scoped to this release.
    pub async fn artifacts(
        &self,
        options: Option<ListArtifactsOptions>,
    ) -> Result<Vec<Artifact>, Error> {
        let mut options = options.unwrap_or_default();
        options.release = Some(self.id.clone());
        Artifact::list(Some(options)).await
    }

    /// List entitlement constraints for this release.
    pub async fn constraints(
        &self,
        options: Option<&PaginationOptions>,
    ) -> Result<Vec<Constraint>, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/constraints", self.id);
        let response = client.get(&endpoint, options).await?;
        let constraints_response: ConstraintsResponse = serde_json::from_value(response.body)?;
        Ok(constraints_response
            .data
            .into_iter()
            .map(Constraint::from)
            .collect())
    }
}

#[cfg(feature = "token")]
impl Release {
    /// Create a new release
    pub async fn create(request: CreateReleaseRequest) -> Result<Release, Error> {
        let client = Client::from_global_config()?;

        let mut attributes = serde_json::Map::new();
        attributes.insert(
            "version".to_string(),
            serde_json::Value::String(request.version),
        );
        attributes.insert(
            "channel".to_string(),
            serde_json::to_value(&request.channel)?,
        );

        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "description", request.description)?;
        insert_optional(&mut attributes, "status", request.status)?;
        insert_optional(&mut attributes, "tag", request.tag)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = serde_json::json!({
            "data": {
                "type": "releases",
                "attributes": attributes,
                "relationships": {
                    "product": {
                        "data": {
                            "type": "products",
                            "id": request.product_id
                        }
                    }
                }
            }
        });

        let response = client.post("releases", Some(&body), None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Update an existing release
    pub async fn update(&self, request: UpdateReleaseRequest) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}", self.id);

        let mut attributes = serde_json::Map::new();
        insert_optional(&mut attributes, "name", request.name)?;
        insert_optional(&mut attributes, "description", request.description)?;
        insert_optional(&mut attributes, "channel", request.channel)?;
        insert_optional(&mut attributes, "tag", request.tag)?;
        insert_optional(&mut attributes, "metadata", request.metadata)?;

        let body = serde_json::json!({
            "data": {
                "type": "releases",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Delete a release
    pub async fn delete(&self) -> Result<(), Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}", self.id);
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Publish a release (DRAFT -> PUBLISHED)
    ///
    /// Makes the release visible to customers
    pub async fn publish(&self) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/actions/publish", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Yank a release (PUBLISHED -> YANKED)
    ///
    /// Makes the release unavailable for distribution
    pub async fn yank(&self) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/actions/yank", self.id);
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }

    /// Attach entitlement constraints to this release.
    pub async fn attach_constraints(
        &self,
        entitlement_ids: &[String],
    ) -> Result<Vec<Constraint>, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/constraints", self.id);
        let data: Vec<serde_json::Value> = entitlement_ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "type": "constraints",
                    "relationships": {
                        "entitlement": {
                            "data": {
                                "type": "entitlements",
                                "id": id
                            }
                        }
                    }
                })
            })
            .collect();
        let body = serde_json::json!({ "data": data });
        let response = client.post(&endpoint, Some(&body), None::<&()>).await?;
        let constraints_response: ConstraintsResponse = serde_json::from_value(response.body)?;
        Ok(constraints_response
            .data
            .into_iter()
            .map(Constraint::from)
            .collect())
    }

    /// Detach constraints from this release by constraint ID.
    pub async fn detach_constraints(&self, constraint_ids: &[String]) -> Result<(), Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/constraints", self.id);
        let data: Vec<serde_json::Value> = constraint_ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "type": "constraints",
                    "id": id
                })
            })
            .collect();
        let body = serde_json::json!({ "data": data });
        client
            .delete::<serde_json::Value, serde_json::Value>(&endpoint, Some(&body))
            .await?;
        Ok(())
    }

    /// Change the package associated with this release.
    pub async fn change_package(&self, package_id: &str) -> Result<Release, Error> {
        let client = Client::from_global_config()?;
        let endpoint = format!("releases/{}/package", self.id);
        let body = serde_json::json!({
            "data": {
                "type": "packages",
                "id": package_id
            }
        });
        let response = client.put(&endpoint, Some(&body), None::<&()>).await?;
        let release_response: ReleaseResponse = serde_json::from_value(response.body)?;
        Ok(Release::from(release_response.data))
    }
}

impl Constraint {
    fn from(data: KeygenResponseData<ConstraintAttributes>) -> Self {
        let entitlement = data
            .relationships
            .other
            .get("entitlement")
            .and_then(|value| serde_json::from_value::<KeygenRelationship>(value.clone()).ok())
            .and_then(|relationship| relationship.data.map(|d| d.id));

        Self {
            id: data.id,
            created: data.attributes.created,
            updated: data.attributes.updated,
            account_id: data.relationships.account_id(),
            entitlement_id: entitlement,
            release_id: data
                .relationships
                .release
                .as_ref()
                .and_then(|rel| rel.data.as_ref().map(|d| d.id.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ApiErrorKind;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };
    use mockito::{mock, server_url};
    use serde_json::json;

    fn create_test_client() -> Client {
        Client::new(ClientOptions::from(KeygenConfig {
            api_url: server_url(),
            account: "test_account".to_string(),
            license_key: Some("test-license-key".to_string()),
            ..Default::default()
        }))
        .expect("Failed to create test client")
    }

    #[test]
    fn test_release_from_response_data() {
        let release_data = KeygenResponseData {
            id: "test-release-id".to_string(),
            r#type: "releases".to_string(),
            attributes: ReleaseAttributes {
                name: Some("v1.0.0".to_string()),
                description: Some("Initial release".to_string()),
                version: "1.0.0".to_string(),
                semver: Some(Semver {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: None,
                    build: None,
                }),
                channel: ReleaseChannel::Stable,
                status: ReleaseStatus::Published,
                tag: Some("v1.0.0".to_string()),
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships {
                product: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "products".to_string(),
                        id: "test-product-id".to_string(),
                    }),
                    links: None,
                }),
                account: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "accounts".to_string(),
                        id: "test-account-id".to_string(),
                    }),
                    links: None,
                }),
                ..Default::default()
            },
        };

        let release = Release::from(release_data);

        assert_eq!(release.id, "test-release-id");
        assert_eq!(release.version, "1.0.0");
        assert_eq!(release.channel, ReleaseChannel::Stable);
        assert_eq!(release.status, ReleaseStatus::Published);
        assert_eq!(release.product_id, Some("test-product-id".to_string()));
        assert_eq!(release.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_release_without_relationships() {
        let release_data = KeygenResponseData {
            id: "test-release-id".to_string(),
            r#type: "releases".to_string(),
            attributes: ReleaseAttributes {
                name: None,
                description: None,
                version: "1.0.0-beta.1".to_string(),
                semver: Some(Semver {
                    major: 1,
                    minor: 0,
                    patch: 0,
                    prerelease: Some("beta.1".to_string()),
                    build: None,
                }),
                channel: ReleaseChannel::Beta,
                status: ReleaseStatus::Draft,
                tag: None,
                metadata: None,
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships::default(),
        };

        let release = Release::from(release_data);

        assert_eq!(release.id, "test-release-id");
        assert_eq!(release.channel, ReleaseChannel::Beta);
        assert_eq!(release.status, ReleaseStatus::Draft);
        assert_eq!(release.product_id, None);
        assert_eq!(release.account_id, None);
    }

    #[test]
    fn test_release_channel_serialization() {
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Stable).unwrap(),
            "\"stable\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Rc).unwrap(),
            "\"rc\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Beta).unwrap(),
            "\"beta\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Alpha).unwrap(),
            "\"alpha\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseChannel::Dev).unwrap(),
            "\"dev\""
        );
    }

    #[test]
    fn test_release_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ReleaseStatus::Draft).unwrap(),
            "\"DRAFT\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseStatus::Published).unwrap(),
            "\"PUBLISHED\""
        );
        assert_eq!(
            serde_json::to_string(&ReleaseStatus::Yanked).unwrap(),
            "\"YANKED\""
        );
    }

    #[test]
    fn test_list_releases_options_serialization() {
        let options = ListReleasesOptions {
            channel: Some(ReleaseChannel::Dev),
            limit: Some(20),
            ..Default::default()
        };

        let query = serde_urlencoded::to_string(&options).unwrap();
        println!("Query string: {}", query);
        assert!(query.contains("channel=dev"));
        assert!(query.contains("limit=20"));
        // Verify None values are not included
        assert!(!query.contains("page"));
        assert!(!query.contains("status"));
    }

    #[tokio::test]
    async fn test_artifact_redirect_returns_location_and_artifact() {
        let _m = mock("GET", "/v1/releases/release-1/artifacts/app.zip")
            .match_header("authorization", "License test-license-key")
            .with_status(303)
            .with_header("content-type", "application/vnd.api+json")
            .with_header("location", "https://s3.example.com/app.zip")
            .with_body(
                json!({
                    "data": {
                        "id": "artifact-1",
                        "type": "artifacts",
                        "attributes": {
                            "filename": "app.zip",
                            "filesize": 1024,
                            "checksum": "Y2hlY2tzdW0",
                            "signature": "c2lnbmF0dXJl",
                            "status": "UPLOADED",
                            "created": "2024-01-01T00:00:00Z",
                            "updated": "2024-01-01T00:00:00Z"
                        },
                        "relationships": {}
                    }
                })
                .to_string(),
            )
            .create();

        let (location, artifact) =
            artifact_redirect_with_client(&create_test_client(), "release-1", "app.zip")
                .await
                .expect("redirect should succeed");

        assert_eq!(location, "https://s3.example.com/app.zip");
        assert_eq!(artifact.id, "artifact-1");
        assert_eq!(artifact.checksum.as_deref(), Some("Y2hlY2tzdW0"));
        assert_eq!(artifact.signature.as_deref(), Some("c2lnbmF0dXJl"));
    }

    #[tokio::test]
    async fn test_artifact_redirect_returns_api_error_when_artifact_is_missing() {
        let _m = mock("GET", "/v1/releases/release-1/artifacts/missing.zip")
            .with_status(404)
            .with_header("content-type", "application/vnd.api+json")
            .with_body(
                json!({
                    "errors": [{
                        "title": "Not found",
                        "detail": "The requested artifact was not found",
                        "code": "NOT_FOUND"
                    }]
                })
                .to_string(),
            )
            .create();

        let err = artifact_redirect_with_client(&create_test_client(), "release-1", "missing.zip")
            .await
            .expect_err("missing artifact should be an error");

        assert!(matches!(err, Error::Api(_)));
        assert_eq!(err.api_error_kind(), ApiErrorKind::NotFound);
    }

    #[test]
    fn test_upgrade_request_defaults() {
        let config = KeygenConfig {
            product: "config-product".to_string(),
            package: "config-package".to_string(),
            ..Default::default()
        };
        let request = ReleaseUpgradeRequest {
            product: Some("other-product".to_string()),
            channel: Some(ReleaseChannel::Beta),
            ..Default::default()
        };

        let query = serde_urlencoded::to_string(ReleaseUpgradeRequest::with_defaults(
            Some(&request),
            &config,
        ))
        .unwrap();

        assert!(query.contains("product=other-product"));
        assert!(query.contains("package=config-package"));
        assert!(query.contains("channel=beta"));
        assert!(!query.contains("constraint"));
    }
}
