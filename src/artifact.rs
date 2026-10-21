use crate::client::Client;
use crate::errors::Error;
use crate::keygen_client::KeygenClient;
use crate::pagination::{list_query, ListDocument, Page};
use crate::KeygenResponseData;
#[cfg(not(target_arch = "wasm32"))]
use base64::{
    alphabet,
    engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
#[cfg(not(target_arch = "wasm32"))]
use ed25519_dalek::{Signature, VerifyingKey};
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use futures::TryStreamExt;
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use http_body::Frame;
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use http_body_util::StreamBody;
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use reqwest::header::CONTENT_LENGTH;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use sha2::{Digest, Sha512};
use std::collections::HashMap;
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use std::{path::Path, time::Duration};
#[cfg(not(target_arch = "wasm32"))]
use subtle::ConstantTimeEq;
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use tokio::io::AsyncReadExt;
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
use tokio_util::io::ReaderStream;

/// Base64 of artifact checksums and signatures: `keygen upload` writes them unpadded
/// by default, so encode without padding and accept both when decoding.
#[cfg(not(target_arch = "wasm32"))]
const ARTIFACT_BASE64: GeneralPurpose = GeneralPurpose::new(
    &alphabet::STANDARD,
    GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

/// Artifact status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactStatus {
    Waiting,
    Uploaded,
    Failed,
    Yanked,
}

/// Artifact attributes from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactAttributes {
    pub filename: String,
    pub filetype: Option<String>,
    pub filesize: Option<u64>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: ArtifactStatus,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    #[serde(rename = "yanked")]
    pub yanked_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ArtifactResponse {
    pub data: KeygenResponseData<ArtifactAttributes>,
}

/// Request to create a new artifact
#[cfg(feature = "token")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateArtifactRequest {
    /// The filename of the artifact
    pub filename: String,
    /// Associated release ID
    pub release_id: String,
    /// Optional: File type/extension
    pub filetype: Option<String>,
    /// Optional: File size in bytes
    pub filesize: Option<u64>,
    /// Optional: Target platform (e.g., "darwin", "linux", "win32")
    pub platform: Option<String>,
    /// Optional: Target architecture (e.g., "amd64", "arm64")
    pub arch: Option<String>,
    /// Optional: Ed25519ph signature of the file, base64-encoded, with the product ID
    /// as context (as `keygen upload --signing-key` produces). Required for downloads
    /// that verify signatures with [`ArtifactDownloadOptions::public_key`](crate::release::ArtifactDownloadOptions::public_key).
    pub signature: Option<String>,
    /// Optional: Checksum, base64-encoded SHA-512 digest of the file
    pub checksum: Option<String>,
    /// Optional: Custom metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to update an existing artifact
#[cfg(feature = "token")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateArtifactRequest {
    pub filename: Option<String>,
    pub filetype: Option<String>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Options for listing artifacts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListArtifactsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    #[serde(rename = "page[size]", skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(rename = "page[number]", skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,
    /// Cursor returned by [`Page::next_cursor`](crate::pagination::Page::next_cursor).
    /// Ignored when `page_number` is set.
    #[serde(rename = "page[cursor]", skip_serializing_if = "Option::is_none")]
    pub page_cursor: Option<String>,
    /// Filter by release ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    /// Filter by product ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    /// Filter by channel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Filter by platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    /// Filter by architecture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    /// Filter by filetype
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filetype: Option<String>,
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArtifactStatus>,
}

/// An artifact represents a distributable file associated with a release
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub filename: String,
    pub filetype: Option<String>,
    pub filesize: Option<u64>,
    pub platform: Option<String>,
    pub arch: Option<String>,
    pub signature: Option<String>,
    pub checksum: Option<String>,
    pub status: ArtifactStatus,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created: String,
    pub updated: String,
    pub yanked_at: Option<String>,
    pub release_id: Option<String>,
    pub product_id: Option<String>,
    pub account_id: Option<String>,
}

impl Artifact {
    pub(crate) fn from(data: KeygenResponseData<ArtifactAttributes>) -> Artifact {
        Artifact {
            id: data.id,
            filename: data.attributes.filename,
            filetype: data.attributes.filetype,
            filesize: data.attributes.filesize,
            platform: data.attributes.platform,
            arch: data.attributes.arch,
            signature: data.attributes.signature,
            checksum: data.attributes.checksum,
            status: data.attributes.status,
            metadata: data.attributes.metadata,
            created: data.attributes.created,
            updated: data.attributes.updated,
            yanked_at: data.attributes.yanked_at,
            release_id: data
                .relationships
                .release
                .as_ref()
                .and_then(|r| r.data.as_ref().map(|d| d.id.clone())),
            product_id: data
                .relationships
                .product
                .as_ref()
                .and_then(|p| p.data.as_ref().map(|d| d.id.clone())),
            account_id: data
                .relationships
                .account
                .as_ref()
                .and_then(|a| a.data.as_ref().map(|d| d.id.clone())),
        }
    }

    /// List artifacts with optional filtering and pagination
    pub async fn list(options: Option<ListArtifactsOptions>) -> Result<Vec<Artifact>, Error> {
        let page = Self::list_with_client(&Client::from_global_config()?, options.as_ref()).await?;
        Ok(page.into_items())
    }

    pub(crate) async fn list_with_client(
        client: &Client,
        options: Option<&ListArtifactsOptions>,
    ) -> Result<Page<Artifact>, Error> {
        let query = list_query(options)?;
        let response = client.get("artifacts", query.as_ref()).await?;
        let document: ListDocument<ArtifactAttributes> = serde_json::from_value(response.body)?;
        Ok(document.into_page(Artifact::from))
    }

    /// Get an artifact by ID
    pub async fn get(id: &str) -> Result<Artifact, Error> {
        Self::get_with_client(&Client::from_global_config()?, id).await
    }

    pub(crate) async fn get_with_client(client: &Client, id: &str) -> Result<Artifact, Error> {
        let endpoint = format!("artifacts/{id}");
        let response = client.get(&endpoint, None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }
}

/// Parse a hex Ed25519 public key generated by `keygen genkey`.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn parse_signing_public_key(public_key: &str) -> Result<VerifyingKey, Error> {
    let bytes: [u8; 32] = hex::decode(public_key)
        .map_err(|e| Error::ArtifactPublicKeyInvalid(e.to_string()))?
        .try_into()
        .map_err(|_| Error::ArtifactPublicKeyInvalid("expected 32 bytes".to_string()))?;
    VerifyingKey::from_bytes(&bytes).map_err(|e| Error::ArtifactPublicKeyInvalid(e.to_string()))
}

/// Incrementally verifies an artifact file, as uploaded by `keygen upload`:
/// the Ed25519ph signature when a public key is given, and the SHA-512 checksum.
///
/// Feed the file with [`update`](Self::update) and call [`finish`](Self::finish);
/// the data must not be used before `finish` returns `Ok`.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct ArtifactVerifier {
    hasher: Sha512,
    checksum: Option<Vec<u8>>,
    signature: Option<(VerifyingKey, Signature, String)>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ArtifactVerifier {
    /// `public_key` is the hex key from `keygen genkey`, not the account key.
    /// `context` is the product ID the artifact was signed for.
    ///
    /// With a public key the artifact must be signed ([`Error::ArtifactSignatureMissing`]);
    /// the signature covers the SHA-512 of the file, so the checksum is then optional
    /// but still checked if present. Without a public key the checksum is required
    /// ([`Error::ArtifactChecksumMissing`]).
    pub(crate) fn new(
        artifact: &Artifact,
        public_key: Option<&str>,
        context: &str,
    ) -> Result<Self, Error> {
        let signature = match public_key {
            Some(public_key) => {
                let key = parse_signing_public_key(public_key)?;
                let signature = artifact
                    .signature
                    .as_deref()
                    .ok_or(Error::ArtifactSignatureMissing)?;
                let signature = ARTIFACT_BASE64
                    .decode(signature)
                    .ok()
                    .and_then(|bytes| Signature::from_slice(&bytes).ok())
                    .ok_or_else(|| {
                        Error::ArtifactSignatureInvalid("malformed signature".to_string())
                    })?;
                Some((key, signature, context.to_string()))
            }
            None => None,
        };

        let checksum = match (artifact.checksum.as_deref(), &signature) {
            (Some(checksum), _) => Some(
                ARTIFACT_BASE64
                    .decode(checksum)
                    .map_err(|_| Error::ArtifactChecksumMismatch)?,
            ),
            (None, Some(_)) => None,
            (None, None) => return Err(Error::ArtifactChecksumMissing),
        };

        Ok(Self {
            hasher: Sha512::new(),
            checksum,
            signature,
        })
    }

    pub(crate) fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    pub(crate) fn finish(self) -> Result<(), Error> {
        if let Some(checksum) = &self.checksum {
            let digest = self.hasher.clone().finalize();
            if !bool::from(digest.as_slice().ct_eq(checksum)) {
                return Err(Error::ArtifactChecksumMismatch);
            }
        }

        if let Some((key, signature, context)) = &self.signature {
            // Ed25519ph over SHA-512 of the file, as `keygen upload` and keygen-go do.
            key.verify_prehashed(self.hasher, Some(context.as_bytes()), signature)
                .map_err(|e| Error::ArtifactSignatureInvalid(e.to_string()))?;
        }

        Ok(())
    }
}

#[cfg(feature = "token")]
fn create_artifact_body(request: CreateArtifactRequest) -> Result<serde_json::Value, Error> {
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "filename".to_string(),
        serde_json::Value::String(request.filename),
    );

    if let Some(filetype) = request.filetype {
        attributes.insert("filetype".to_string(), serde_json::Value::String(filetype));
    }
    if let Some(filesize) = request.filesize {
        attributes.insert("filesize".to_string(), serde_json::json!(filesize));
    }
    if let Some(platform) = request.platform {
        attributes.insert("platform".to_string(), serde_json::Value::String(platform));
    }
    if let Some(arch) = request.arch {
        attributes.insert("arch".to_string(), serde_json::Value::String(arch));
    }
    if let Some(signature) = request.signature {
        attributes.insert(
            "signature".to_string(),
            serde_json::Value::String(signature),
        );
    }
    if let Some(checksum) = request.checksum {
        attributes.insert("checksum".to_string(), serde_json::Value::String(checksum));
    }
    if let Some(metadata) = request.metadata {
        attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
    }

    Ok(serde_json::json!({
        "data": {
            "type": "artifacts",
            "attributes": attributes,
            "relationships": {
                "release": {
                    "data": {
                        "type": "releases",
                        "id": request.release_id
                    }
                }
            }
        }
    }))
}

/// An artifact created by Keygen that is waiting for its file to be uploaded.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
struct ArtifactUpload {
    /// The created artifact, usually in the `WAITING` status.
    artifact: Artifact,
    /// Presigned URL to `PUT` the artifact file to.
    url: String,
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
impl std::fmt::Debug for ArtifactUpload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The URL carries a signature: never print it, even from debug logs.
        f.debug_struct("ArtifactUpload")
            .field("artifact", &self.artifact)
            .field("url", &"<redacted>")
            .finish()
    }
}

/// Upload progress callback: uploaded bytes and the file size.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
pub type UploadProgress = Box<dyn FnMut(u64, u64) + Send>;

/// Options for [`Artifact::create_and_upload`].
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
#[derive(Default)]
pub struct ArtifactUploadOptions {
    /// Called with the uploaded byte count and the file size while the file is sent.
    /// Not called while the checksum is computed before the upload.
    pub on_progress: Option<UploadProgress>,
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
impl std::fmt::Debug for ArtifactUploadOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactUploadOptions")
            .field("on_progress", &self.on_progress.is_some())
            .finish()
    }
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
impl ArtifactUpload {
    /// Stream a file from disk to the presigned URL.
    ///
    /// The request is sent without Keygen headers, since the storage provider
    /// rejects requests that carry them.
    async fn upload_file(
        &self,
        path: &Path,
        on_progress: Option<UploadProgress>,
    ) -> Result<(), Error> {
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|e| io_error(path, e))?;
        let filesize = file.metadata().await.map_err(|e| io_error(path, e))?.len();

        // Progress counts bytes handed to the HTTP client, not acknowledged by storage.
        // The mutex only makes the stream `Sync` for `Body::wrap`; it is never contended.
        let progress = std::sync::Mutex::new(on_progress);
        let mut uploaded = 0u64;
        let chunks = ReaderStream::with_capacity(file, 256 * 1024)
            .inspect_ok(move |chunk| {
                uploaded += chunk.len() as u64;
                if let Some(on_progress) = progress.lock().unwrap().as_mut() {
                    on_progress(uploaded, filesize);
                }
            })
            .map_ok(Frame::data);

        let request = upload_client()?
            .put(&self.url)
            .header(CONTENT_LENGTH, filesize)
            .body(reqwest::Body::wrap(StreamBody::new(chunks)));
        send_upload(request).await
    }
}

/// Client for uploads to the storage provider: no Keygen headers and no total
/// timeout, since large files can take minutes to send. Only the TCP connect is
/// bounded. `read_timeout` cannot be used here: it runs until the response
/// headers arrive, so it would cap the whole upload instead of a stalled one.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn upload_client() -> Result<reqwest::Client, Error> {
    reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(30))
        // Detects a dead peer. A server that keeps the socket open but stops
        // reading still hangs the upload; only a timeout or the caller can stop it.
        .tcp_keepalive(Duration::from_secs(30))
        .build()
        .map_err(|e| Error::UnexpectedError(format!("Failed to build HTTP client: {e}")))
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
async fn send_upload(request: reqwest::RequestBuilder) -> Result<(), Error> {
    // A presigned URL is a secret. `reqwest::Error` includes the URL in its
    // `Display`, so strip it before wrapping the error.
    let response = request
        .send()
        .await
        .map_err(|e| Error::from(e.without_url()))?;
    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        let summary = storage_error_summary(&body);
        let detail = if summary.is_empty() {
            String::new()
        } else {
            format!(": {summary}")
        };
        return Err(Error::UnexpectedError(format!(
            "Artifact upload failed with status {status}{detail}"
        )));
    }
    Ok(())
}

/// Summarize a storage error without echoing the raw body.
///
/// S3-style providers answer with XML. Their errors can repeat the request
/// signature (`SignatureProvided`, `StringToSign`), so only `<Code>` and
/// `<Message>` are kept. Any other body is dropped: a proxy error has no useful
/// signature-free part, and echoing it could leak one in an unexpected format.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn storage_error_summary(body: &str) -> String {
    match (xml_tag(body, "Code"), xml_tag(body, "Message")) {
        (Some(code), Some(message)) => format!("{code}: {message}"),
        (Some(code), None) => code,
        (None, Some(message)) => message,
        (None, None) => String::new(),
    }
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn xml_tag(body: &str, tag: &str) -> Option<String> {
    let value = body
        .split_once(&format!("<{tag}>"))?
        .1
        .split_once(&format!("</{tag}>"))?
        .0
        .trim();
    (!value.is_empty()).then(|| short_error_body(value))
}

/// Keep a short prefix of a response body, cut on a character boundary.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn short_error_body(body: &str) -> String {
    const MAX_BYTES: usize = 512;
    let body = body.trim();
    if body.len() <= MAX_BYTES {
        return body.to_string();
    }
    let mut end = MAX_BYTES;
    while !body.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &body[..end])
}

/// Delete an artifact after a failed upload.
///
/// Cleanup is best-effort: the upload error is the one the caller sees, so a
/// failed delete is returned for the caller to append to that error.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
async fn delete_artifact_best_effort(client: &Client, id: &str) -> Result<(), String> {
    let endpoint = format!("artifacts/{id}");
    client
        .delete::<(), ()>(&endpoint, None)
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Keep the original error, but say so when the artifact was left behind.
///
/// On a failed cleanup the variant is flattened to `UnexpectedError`, so callers
/// must not depend on the variant to classify an upload error. Telling the user
/// about the leftover artifact is worth that. Keeping the type would need a new
/// `Error` variant; retries do not exist yet.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn add_cleanup_note(err: Error, id: &str, cleanup: Result<(), String>) -> Error {
    match cleanup {
        Ok(()) => err,
        Err(cleanup_error) => Error::UnexpectedError(format!(
            "{err} (artifact {id} was left behind, delete it manually: {cleanup_error})"
        )),
    }
}

/// Error for an upload that succeeded but whose status could not be read.
///
/// The artifact id is in the message, so the caller can poll it instead of
/// uploading the file again under the same name.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn upload_status_error(id: &str, err: &Error) -> Error {
    Error::UnexpectedError(format!(
        "Artifact {id} was uploaded, but its status could not be fetched: {err}. \
         Do not upload again: fetch it with Artifact::get."
    ))
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
fn io_error(path: &Path, error: std::io::Error) -> Error {
    Error::UnexpectedError(format!("Failed to read {}: {error}", path.display()))
}

/// SHA-512 checksum of a file, in the format [`ArtifactVerifier`] and keygen-go
/// verify downloaded artifacts against.
#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
pub(crate) async fn file_checksum(path: &Path) -> Result<String, Error> {
    let mut file = tokio::fs::File::open(path)
        .await
        .map_err(|e| io_error(path, e))?;
    let mut hasher = Sha512::new();
    let mut buffer = vec![0; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .await
            .map_err(|e| io_error(path, e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(ARTIFACT_BASE64.encode(hasher.finalize()))
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
async fn create_upload_with_client(
    client: &Client,
    request: CreateArtifactRequest,
) -> Result<ArtifactUpload, Error> {
    let body = create_artifact_body(request)?;
    let response = client
        .post::<_, serde_json::Value, ()>("artifacts", Some(&body), None)
        .await?;

    // The artifact exists from this point on. Every error below must delete it,
    // otherwise it stays WAITING and blocks retrying with the same filename.
    let artifact_id = response
        .body
        .pointer("/data/id")
        .and_then(|id| id.as_str())
        .map(str::to_string);
    let artifact_response: ArtifactResponse = match serde_json::from_value(response.body) {
        Ok(response) => response,
        Err(err) => {
            let Some(id) = artifact_id.as_deref() else {
                return Err(err.into());
            };
            let cleanup = delete_artifact_best_effort(client, id).await;
            return Err(add_cleanup_note(err.into(), id, cleanup));
        }
    };
    let artifact = Artifact::from(artifact_response.data);

    let url = match response
        .headers
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
    {
        Some(url) => url.to_string(),
        None => {
            let err = Error::UnexpectedError("Missing upload Location header".to_string());
            let cleanup = delete_artifact_best_effort(client, &artifact.id).await;
            return Err(add_cleanup_note(err, &artifact.id, cleanup));
        }
    };

    Ok(ArtifactUpload { artifact, url })
}

#[cfg(all(feature = "token", not(target_arch = "wasm32")))]
pub(crate) async fn create_and_upload_with_client(
    client: &Client,
    mut request: CreateArtifactRequest,
    path: &Path,
    options: Option<ArtifactUploadOptions>,
) -> Result<Artifact, Error> {
    let options = options.unwrap_or_default();
    let metadata = tokio::fs::metadata(path)
        .await
        .map_err(|e| io_error(path, e))?;
    if !metadata.is_file() {
        return Err(Error::UnexpectedError(format!(
            "Artifact path {} is not a file",
            path.display()
        )));
    }
    let actual_size = metadata.len();
    match request.filesize {
        Some(filesize) if filesize != actual_size => {
            return Err(Error::UnexpectedError(format!(
                "Artifact filesize {filesize} does not match {} ({actual_size} bytes)",
                path.display()
            )));
        }
        Some(_) => {}
        None => request.filesize = Some(actual_size),
    }
    if request.checksum.is_none() {
        request.checksum = Some(file_checksum(path).await?);
    }

    let upload = create_upload_with_client(client, request).await?;
    let endpoint = format!("artifacts/{}", upload.artifact.id);
    if let Err(err) = upload.upload_file(path, options.on_progress).await {
        // Do not leave a WAITING artifact behind: it would block retrying with
        // the same filename. The upload error is the one the caller sees.
        let cleanup = delete_artifact_best_effort(client, &upload.artifact.id).await;
        return Err(add_cleanup_note(err, &upload.artifact.id, cleanup));
    }

    let response = match client.get(&endpoint, None::<&()>).await {
        Ok(response) => response,
        Err(err) => return Err(upload_status_error(&upload.artifact.id, &err)),
    };
    let artifact_response: ArtifactResponse = match serde_json::from_value(response.body) {
        Ok(response) => response,
        Err(err) => return Err(upload_status_error(&upload.artifact.id, &err.into())),
    };
    Ok(Artifact::from(artifact_response.data))
}

#[cfg(feature = "token")]
impl Artifact {
    /// Create a new artifact
    ///
    /// Only the artifact record is created: Keygen returns the upload URL in a
    /// redirect, which is discarded here. Use [`Artifact::create_and_upload`] to
    /// upload the file.
    pub async fn create(request: CreateArtifactRequest) -> Result<Artifact, Error> {
        Self::create_with_client(&Client::from_global_config()?, request).await
    }

    pub(crate) async fn create_with_client(
        client: &Client,
        request: CreateArtifactRequest,
    ) -> Result<Artifact, Error> {
        let body = create_artifact_body(request)?;
        let response = client.post("artifacts", Some(&body), None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// Create a new artifact and upload the file at `path`.
    ///
    /// `filesize` and `checksum` are computed from the file unless set in the request.
    /// A `filesize` that differs from the file on disk is an error, and nothing is
    /// created. The file is read twice (for the checksum, then for the upload), so it
    /// must not change in between. If the upload fails, the created artifact is deleted.
    ///
    /// The file is not signed: set [`CreateArtifactRequest::signature`] for downloads
    /// that verify signatures.
    ///
    /// Returns the artifact fetched after the upload, usually still `WAITING`:
    /// Keygen processes the file with a delay (a few minutes on Keygen Cloud),
    /// and until the status is `UPLOADED` the artifact cannot be downloaded.
    /// Check the status with [`Artifact::get`].
    ///
    /// If the file is uploaded but its status cannot be fetched, the error names
    /// the artifact: fetch it with [`Artifact::get`] instead of uploading again.
    ///
    /// Dropping the future before the upload finishes leaves the artifact in
    /// `WAITING`. Delete it with [`Artifact::delete`] before retrying with the
    /// same filename.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn create_and_upload(
        request: CreateArtifactRequest,
        path: impl AsRef<Path>,
        options: Option<ArtifactUploadOptions>,
    ) -> Result<Artifact, Error> {
        let client = Client::from_global_config()?;
        create_and_upload_with_client(&client, request, path.as_ref(), options).await
    }

    /// Update an existing artifact
    pub async fn update(&self, request: UpdateArtifactRequest) -> Result<Artifact, Error> {
        Self::update_with_client(&Client::from_global_config()?, &self.id, request).await
    }

    pub(crate) async fn update_with_client(
        client: &Client,
        id: &str,
        request: UpdateArtifactRequest,
    ) -> Result<Artifact, Error> {
        let endpoint = format!("artifacts/{id}");

        let mut attributes = serde_json::Map::new();
        if let Some(filename) = request.filename {
            attributes.insert("filename".to_string(), serde_json::Value::String(filename));
        }
        if let Some(filetype) = request.filetype {
            attributes.insert("filetype".to_string(), serde_json::Value::String(filetype));
        }
        if let Some(platform) = request.platform {
            attributes.insert("platform".to_string(), serde_json::Value::String(platform));
        }
        if let Some(arch) = request.arch {
            attributes.insert("arch".to_string(), serde_json::Value::String(arch));
        }
        if let Some(signature) = request.signature {
            attributes.insert(
                "signature".to_string(),
                serde_json::Value::String(signature),
            );
        }
        if let Some(checksum) = request.checksum {
            attributes.insert("checksum".to_string(), serde_json::Value::String(checksum));
        }
        if let Some(metadata) = request.metadata {
            attributes.insert("metadata".to_string(), serde_json::to_value(metadata)?);
        }

        let body = serde_json::json!({
            "data": {
                "type": "artifacts",
                "attributes": attributes
            }
        });

        let response = client.patch(&endpoint, Some(&body), None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }

    /// Delete an artifact
    pub async fn delete(&self) -> Result<(), Error> {
        Self::delete_with_client(&Client::from_global_config()?, &self.id).await
    }

    pub(crate) async fn delete_with_client(client: &Client, id: &str) -> Result<(), Error> {
        let endpoint = format!("artifacts/{id}");
        client.delete::<(), ()>(&endpoint, None::<&()>).await?;
        Ok(())
    }

    /// Yank an artifact (make it unavailable for download)
    pub async fn yank(&self) -> Result<Artifact, Error> {
        Self::yank_with_client(&Client::from_global_config()?, &self.id).await
    }

    pub(crate) async fn yank_with_client(client: &Client, id: &str) -> Result<Artifact, Error> {
        let endpoint = format!("artifacts/{id}/actions/yank");
        let response = client.post(&endpoint, None::<&()>, None::<&()>).await?;
        let artifact_response: ArtifactResponse = serde_json::from_value(response.body)?;
        Ok(Artifact::from(artifact_response.data))
    }
}

/// Artifact operations bound to one [`KeygenClient`].
pub struct ArtifactService<'a> {
    client: &'a KeygenClient,
}

impl<'a> ArtifactService<'a> {
    pub(crate) fn new(client: &'a KeygenClient) -> Self {
        Self { client }
    }

    #[cfg(feature = "token")]
    pub async fn create(&self, request: CreateArtifactRequest) -> Result<Artifact, Error> {
        Artifact::create_with_client(&self.client.transport, request).await
    }

    /// Create an artifact and upload the file at `path`.
    ///
    /// Same as [`Artifact::create_and_upload`], but uses this client instead of
    /// the global configuration.
    #[cfg(all(feature = "token", not(target_arch = "wasm32")))]
    pub async fn create_and_upload(
        &self,
        request: CreateArtifactRequest,
        path: impl AsRef<Path>,
        options: Option<ArtifactUploadOptions>,
    ) -> Result<Artifact, Error> {
        create_and_upload_with_client(&self.client.transport, request, path.as_ref(), options).await
    }

    pub async fn list(
        &self,
        options: Option<&ListArtifactsOptions>,
    ) -> Result<Page<Artifact>, Error> {
        Artifact::list_with_client(&self.client.transport, options).await
    }

    pub async fn get(&self, id: &str) -> Result<Artifact, Error> {
        Artifact::get_with_client(&self.client.transport, id).await
    }

    #[cfg(feature = "token")]
    pub async fn update(
        &self,
        id: &str,
        request: UpdateArtifactRequest,
    ) -> Result<Artifact, Error> {
        Artifact::update_with_client(&self.client.transport, id, request).await
    }

    #[cfg(feature = "token")]
    pub async fn delete(&self, id: &str) -> Result<(), Error> {
        Artifact::delete_with_client(&self.client.transport, id).await
    }

    #[cfg(feature = "token")]
    pub async fn yank(&self, id: &str) -> Result<Artifact, Error> {
        Artifact::yank_with_client(&self.client.transport, id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeygenRelationship, KeygenRelationshipData, KeygenRelationships, KeygenResponseData,
    };

    #[test]
    fn test_artifact_from_response_data() {
        let artifact_data = KeygenResponseData {
            id: "test-artifact-id".to_string(),
            r#type: "artifacts".to_string(),
            attributes: ArtifactAttributes {
                filename: "app-1.0.0.dmg".to_string(),
                filetype: Some("dmg".to_string()),
                filesize: Some(10485760),
                platform: Some("darwin".to_string()),
                arch: Some("amd64".to_string()),
                signature: None,
                checksum: Some("abc123".to_string()),
                status: ArtifactStatus::Uploaded,
                metadata: Some(HashMap::new()),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-01T00:00:00Z".to_string(),
                yanked_at: None,
            },
            relationships: KeygenRelationships {
                release: Some(KeygenRelationship {
                    data: Some(KeygenRelationshipData {
                        r#type: "releases".to_string(),
                        id: "test-release-id".to_string(),
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

        let artifact = Artifact::from(artifact_data);

        assert_eq!(artifact.id, "test-artifact-id");
        assert_eq!(artifact.filename, "app-1.0.0.dmg");
        assert_eq!(artifact.filetype, Some("dmg".to_string()));
        assert_eq!(artifact.filesize, Some(10485760));
        assert_eq!(artifact.platform, Some("darwin".to_string()));
        assert_eq!(artifact.arch, Some("amd64".to_string()));
        assert_eq!(artifact.status, ArtifactStatus::Uploaded);
        assert_eq!(artifact.release_id, Some("test-release-id".to_string()));
        assert_eq!(artifact.account_id, Some("test-account-id".to_string()));
    }

    #[test]
    fn test_artifact_status_serialization() {
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Waiting).unwrap(),
            "\"WAITING\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Uploaded).unwrap(),
            "\"UPLOADED\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Failed).unwrap(),
            "\"FAILED\""
        );
        assert_eq!(
            serde_json::to_string(&ArtifactStatus::Yanked).unwrap(),
            "\"YANKED\""
        );
    }

    // Reproduce with calculateChecksum/calculateSignature from keygen-cli cmd/upload.go
    // (commit 3fa3a56), curve25519-voi v0.0.0-20220328075252-7dd334e3daae:
    // ed25519.NewKeyFromSeed(0x01..=0x20), SHA-512 of PAYLOAD, Ed25519ph with
    // Context = CONTEXT, base64.RawStdEncoding. Same vectors as in tests/release_api.rs.
    const PAYLOAD: &[u8] = b"hello keygen";
    const CONTEXT: &str = "test-product-id";
    const PUBLIC_KEY: &str = "79b5562e8fe654f94078b112e8a98ba7901f853ae695bed7e0e3910bad049664";
    const CHECKSUM: &str =
        "dnEi9Lf9kMb2ntwoiG5vSE5tXvJrGo0rLYTMBMcwLP5/TnwwbLgfzp8RuVIRTawUEiuwblnd6laUR0Zzt1BytA";
    const SIGNATURE: &str =
        "YSpVi2Y333gwAyaSGZC67UDdslK8WjbH/QZ1DINDRMseUsNYd6ML++nPUgZqE+q3H1xaQ9y1OdegGH7jzFkPDg";

    fn artifact(checksum: Option<&str>, signature: Option<&str>) -> Artifact {
        Artifact {
            id: "artifact-1".to_string(),
            filename: "app".to_string(),
            filetype: None,
            filesize: Some(PAYLOAD.len() as u64),
            platform: None,
            arch: None,
            signature: signature.map(str::to_string),
            checksum: checksum.map(str::to_string),
            status: ArtifactStatus::Uploaded,
            metadata: None,
            created: "2024-01-01T00:00:00Z".to_string(),
            updated: "2024-01-01T00:00:00Z".to_string(),
            yanked_at: None,
            release_id: None,
            product_id: None,
            account_id: None,
        }
    }

    fn verify(
        artifact: &Artifact,
        data: &[u8],
        public_key: Option<&str>,
        context: &str,
    ) -> Result<(), Error> {
        let mut verifier = ArtifactVerifier::new(artifact, public_key, context)?;
        verifier.update(data);
        verifier.finish()
    }

    #[test]
    fn test_verifier_accepts_go_signed_artifact() {
        verify(
            &artifact(Some(CHECKSUM), Some(SIGNATURE)),
            PAYLOAD,
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect("artifact signed by keygen upload should verify");
    }

    #[test]
    fn test_verifier_accepts_chunked_input_and_padded_base64() {
        let artifact = artifact(
            Some(&format!("{CHECKSUM}==")),
            Some(&format!("{SIGNATURE}==")),
        );
        let mut verifier = ArtifactVerifier::new(&artifact, Some(PUBLIC_KEY), CONTEXT).unwrap();
        for chunk in PAYLOAD.chunks(5) {
            verifier.update(chunk);
        }
        verifier.finish().expect("chunked input should verify");
    }

    #[test]
    fn test_verifier_rejects_checksum_mismatch() {
        let err = verify(
            &artifact(Some(CHECKSUM), None),
            b"hello keygen!",
            None,
            CONTEXT,
        )
        .expect_err("modified payload should fail");
        assert!(matches!(err, Error::ArtifactChecksumMismatch));

        // SHA-256 of PAYLOAD, as `keygen upload --checksum-algorithm sha-256` writes it
        let err = verify(
            &artifact(Some("D9awDuPGxPqXIvc1yl4wSrtl7ge2Epri0MgpMf3lds8"), None),
            PAYLOAD,
            None,
            CONTEXT,
        )
        .expect_err("checksum of another algorithm should fail");
        assert!(matches!(err, Error::ArtifactChecksumMismatch));
    }

    #[test]
    fn test_verifier_rejects_invalid_signature() {
        let other_key = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
        let cases = [(PUBLIC_KEY, "other-product-id"), (other_key, CONTEXT)];
        for (public_key, context) in cases {
            let err = verify(
                &artifact(Some(CHECKSUM), Some(SIGNATURE)),
                PAYLOAD,
                Some(public_key),
                context,
            )
            .expect_err("wrong key or context should fail");
            assert!(matches!(err, Error::ArtifactSignatureInvalid(_)));
        }
    }

    #[test]
    fn test_verifier_requires_checksum_without_public_key() {
        let err = verify(&artifact(None, Some(SIGNATURE)), PAYLOAD, None, CONTEXT).err();
        assert!(matches!(err, Some(Error::ArtifactChecksumMissing)));
    }

    #[test]
    fn test_verifier_requires_signature_with_public_key() {
        for checksum in [Some(CHECKSUM), None] {
            let err = verify(
                &artifact(checksum, None),
                PAYLOAD,
                Some(PUBLIC_KEY),
                CONTEXT,
            )
            .err();
            assert!(matches!(err, Some(Error::ArtifactSignatureMissing)));
        }
    }

    #[test]
    fn test_verifier_signature_alone_is_enough() {
        verify(
            &artifact(None, Some(SIGNATURE)),
            PAYLOAD,
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect("valid signature should verify without a checksum");

        let err = verify(
            &artifact(None, Some(SIGNATURE)),
            b"hello keygen!",
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect_err("modified payload should fail");
        assert!(matches!(err, Error::ArtifactSignatureInvalid(_)));
    }

    #[test]
    fn test_verifier_checks_present_checksum_even_with_valid_signature() {
        let wrong_checksum = ARTIFACT_BASE64.encode(Sha512::digest(b"something else"));
        let err = verify(
            &artifact(Some(&wrong_checksum), Some(SIGNATURE)),
            PAYLOAD,
            Some(PUBLIC_KEY),
            CONTEXT,
        )
        .expect_err("inconsistent checksum should fail");
        assert!(matches!(err, Error::ArtifactChecksumMismatch));
    }

    #[test]
    fn test_verifier_rejects_malformed_public_key() {
        for public_key in ["not-hex", "79b5562e"] {
            let err = verify(
                &artifact(Some(CHECKSUM), Some(SIGNATURE)),
                PAYLOAD,
                Some(public_key),
                CONTEXT,
            )
            .err();
            assert!(matches!(err, Some(Error::ArtifactPublicKeyInvalid(_))));
        }
    }

    #[test]
    fn test_verifier_rejects_malformed_signature_and_checksum() {
        let short_signature = ARTIFACT_BASE64.encode([0u8; 32]);
        for signature in ["not-base64!", short_signature.as_str()] {
            let err = verify(
                &artifact(Some(CHECKSUM), Some(signature)),
                PAYLOAD,
                Some(PUBLIC_KEY),
                CONTEXT,
            )
            .err();
            assert!(matches!(err, Some(Error::ArtifactSignatureInvalid(_))));
        }

        let err = verify(&artifact(Some("not-base64!"), None), PAYLOAD, None, CONTEXT).err();
        assert!(matches!(err, Some(Error::ArtifactChecksumMismatch)));
    }

    #[cfg(all(feature = "token", not(target_arch = "wasm32")))]
    mod upload {
        use super::*;
        use crate::client::ClientOptions;
        use crate::config::KeygenConfig;
        use mockito::{mock, server_url, Matcher};
        use serde_json::json;

        fn create_test_client() -> Client {
            Client::new(ClientOptions::from(KeygenConfig {
                api_url: server_url(),
                account: "test_account".to_string(),
                token: Some("test-token".to_string()),
                ..Default::default()
            }))
            .expect("Failed to create test client")
        }

        fn create_test_keygen_client() -> KeygenClient {
            KeygenClient::new(KeygenConfig {
                api_url: server_url(),
                account: "test_account".to_string(),
                token: Some("test-token".to_string()),
                ..Default::default()
            })
            .expect("Failed to create test client")
        }

        fn artifact_body(id: &str, status: &str) -> String {
            json!({
                "data": {
                    "id": id,
                    "type": "artifacts",
                    "attributes": {
                        "filename": "app.zip",
                        "filesize": 5,
                        "status": status,
                        "created": "2024-01-01T00:00:00Z",
                        "updated": "2024-01-01T00:00:00Z"
                    },
                    "relationships": {}
                }
            })
            .to_string()
        }

        fn create_request() -> CreateArtifactRequest {
            CreateArtifactRequest {
                filename: "app.zip".to_string(),
                release_id: "release-1".to_string(),
                filetype: None,
                filesize: Some(5),
                platform: None,
                arch: None,
                signature: None,
                checksum: None,
                metadata: None,
            }
        }

        fn test_upload(path: &str) -> ArtifactUpload {
            let response: ArtifactResponse =
                serde_json::from_str(&artifact_body("artifact-1", "WAITING")).unwrap();
            ArtifactUpload {
                artifact: Artifact::from(response.data),
                url: format!("{}{path}", server_url()),
            }
        }

        #[tokio::test]
        async fn create_upload_returns_location_without_following_it() {
            let location = format!("{}/s3/create_upload", server_url());
            let target = mock("POST", "/s3/create_upload").expect(0).create();
            let _m = mock("POST", "/v1/artifacts")
                .match_header("authorization", "Bearer test-token")
                .match_body(Matcher::PartialJson(json!({
                    "data": {
                        "attributes": { "filename": "app.zip", "filesize": 5 },
                        "relationships": {
                            "release": { "data": { "type": "releases", "id": "release-1" } }
                        }
                    }
                })))
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", &location)
                .with_body(artifact_body("artifact-1", "WAITING"))
                .create();

            let upload = create_upload_with_client(&create_test_client(), create_request())
                .await
                .expect("create should succeed");

            assert_eq!(upload.url, location);
            assert_eq!(upload.artifact.id, "artifact-1");
            assert_eq!(upload.artifact.status, ArtifactStatus::Waiting);
            target.assert();
        }

        #[tokio::test]
        async fn create_upload_fails_without_location() {
            let _m = mock("POST", "/v1/artifacts")
                .with_status(201)
                .with_header("content-type", "application/vnd.api+json")
                .with_body(artifact_body("artifact-1", "WAITING"))
                .create();
            let delete = mock("DELETE", "/v1/artifacts/artifact-1")
                .with_status(204)
                .create();

            let err = create_upload_with_client(&create_test_client(), create_request())
                .await
                .expect_err("missing Location should be an error");

            assert!(matches!(err, Error::UnexpectedError(_)));
            delete.assert();
        }

        #[tokio::test]
        async fn create_upload_reports_failed_cleanup() {
            let _m = mock("POST", "/v1/artifacts")
                .with_status(201)
                .with_header("content-type", "application/vnd.api+json")
                .with_body(artifact_body("artifact-1", "WAITING"))
                .create();
            let _delete = mock("DELETE", "/v1/artifacts/artifact-1")
                .with_status(500)
                .create();

            let err = create_upload_with_client(&create_test_client(), create_request())
                .await
                .expect_err("missing Location should be an error");

            let text = err.to_string();
            assert!(text.contains("artifact-1"), "{text}");
            assert!(text.contains("left behind"), "{text}");
        }

        #[tokio::test]
        async fn create_upload_deletes_artifact_when_body_is_invalid() {
            let _m = mock("POST", "/v1/artifacts")
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", "https://s3.example.com/upload")
                .with_body(
                    r#"{"data": {"id": "artifact-bad", "type": "artifacts", "attributes": {}}}"#,
                )
                .create();
            let delete = mock("DELETE", "/v1/artifacts/artifact-bad")
                .with_status(204)
                .create();

            let err = create_upload_with_client(&create_test_client(), create_request())
                .await
                .expect_err("malformed artifact body should be an error");

            assert!(matches!(err, Error::JsonError(_)));
            delete.assert();
        }

        #[tokio::test]
        async fn upload_file_streams_file_with_content_length() {
            let path = std::env::temp_dir().join(format!("keygen-rs-{}.bin", uuid::Uuid::new_v4()));
            tokio::fs::write(&path, b"file contents").await.unwrap();
            let m = mock("PUT", "/s3/upload_file")
                .match_header("authorization", Matcher::Missing)
                .match_header("keygen-version", Matcher::Missing)
                .match_header("content-length", "13")
                .match_header("transfer-encoding", Matcher::Missing)
                .match_body("file contents")
                .with_status(200)
                .create();

            let progress = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let recorded = progress.clone();
            let on_progress: UploadProgress = Box::new(move |uploaded, total| {
                recorded.lock().unwrap().push((uploaded, total));
            });

            let result = test_upload("/s3/upload_file")
                .upload_file(&path, Some(on_progress))
                .await;
            tokio::fs::remove_file(&path).await.ok();

            result.expect("upload should succeed");
            m.assert();
            assert_eq!(progress.lock().unwrap().last(), Some(&(13, 13)));
        }

        #[tokio::test]
        async fn upload_file_reports_monotonic_progress_for_multiple_chunks() {
            let size = 700 * 1024;
            let path = temp_file(&vec![0xAB; size]);
            let _m = mock("PUT", "/s3/large_upload")
                .match_header("content-length", Matcher::Exact(size.to_string()))
                .with_status(200)
                .create();

            let progress = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let recorded = progress.clone();
            let on_progress: UploadProgress = Box::new(move |uploaded, total| {
                recorded.lock().unwrap().push((uploaded, total));
            });

            let result = test_upload("/s3/large_upload")
                .upload_file(&path, Some(on_progress))
                .await;
            std::fs::remove_file(&path).ok();
            result.expect("large upload should succeed");

            let progress = progress.lock().unwrap();
            assert!(progress.len() > 1, "one chunk per callback: {progress:?}");
            assert!(
                progress.windows(2).all(|steps| steps[0].0 <= steps[1].0),
                "progress must not go backwards: {progress:?}"
            );
            assert!(
                progress.iter().all(|(_, total)| *total == size as u64),
                "{progress:?}"
            );
            assert_eq!(progress.last(), Some(&(size as u64, size as u64)));
        }

        #[tokio::test]
        async fn create_and_upload_rejects_filesize_mismatch() {
            let path = temp_file(b"hello");
            let mut request = create_request();
            request.filesize = Some(999);

            // No artifact mocks: a mismatch must fail before the POST.
            let err = create_and_upload_with_client(&create_test_client(), request, &path, None)
                .await
                .expect_err("filesize mismatch should be an error");
            std::fs::remove_file(&path).ok();

            let text = err.to_string();
            assert!(text.contains("999"), "{text}");
            assert!(text.contains("(5 bytes)"), "{text}");
        }

        #[tokio::test]
        async fn create_and_upload_rejects_directory() {
            let mut request = create_request();
            request.filesize = None;
            // Checksum supplied, so only the path check can stop the upload.
            request.checksum = Some("provided".to_string());

            // No artifact mocks: a directory must fail before the POST.
            let err = create_and_upload_with_client(
                &create_test_client(),
                request,
                &std::env::temp_dir(),
                None,
            )
            .await
            .expect_err("a directory is not an artifact");

            let text = err.to_string();
            assert!(text.contains("is not a file"), "{text}");
        }

        fn temp_file(contents: &[u8]) -> std::path::PathBuf {
            let path = std::env::temp_dir().join(format!("keygen-rs-{}.bin", uuid::Uuid::new_v4()));
            std::fs::write(&path, contents).unwrap();
            path
        }

        #[tokio::test]
        async fn upload_fails_on_storage_error() {
            let path = temp_file(b"hello");
            let _m = mock("PUT", "/s3/upload_denied")
                .with_status(403)
                .with_body(
                    "<Error><Code>SignatureDoesNotMatch</Code>\
                     <Message>The request signature we calculated does not match</Message>\
                     <SignatureProvided>top-secret-signature</SignatureProvided></Error>",
                )
                .create();

            let result = test_upload("/s3/upload_denied")
                .upload_file(&path, None)
                .await;
            std::fs::remove_file(&path).ok();

            let err = result.expect_err("storage error should be an error");
            let text = err.to_string();
            assert!(text.contains("403"), "{text}");
            assert!(text.contains("SignatureDoesNotMatch"), "{text}");
            assert!(!text.contains("top-secret-signature"), "{text}");
        }

        #[tokio::test]
        async fn upload_error_without_xml_hides_body() {
            let path = temp_file(b"hello");
            let _m = mock("PUT", "/s3/proxy_error")
                .with_status(502)
                .with_body("<html>proxy said secret-token</html>")
                .create();

            let result = test_upload("/s3/proxy_error")
                .upload_file(&path, None)
                .await;
            std::fs::remove_file(&path).ok();

            let err = result.expect_err("storage error should be an error");
            let text = err.to_string();
            assert!(text.contains("502"), "{text}");
            assert!(!text.contains("secret-token"), "{text}");
        }

        #[tokio::test]
        async fn upload_error_does_not_leak_presigned_url() {
            let path = temp_file(b"hello");
            let mut upload = test_upload("");
            upload.url = "http://127.0.0.1:1/secret.bin?X-Amz-Signature=top-secret".to_string();

            let err = upload
                .upload_file(&path, None)
                .await
                .expect_err("connection to port 1 should fail");
            std::fs::remove_file(&path).ok();

            let text = err.to_string();
            let debug = format!("{err:?}");
            // The presigned query must never leak, in `Display` or in `Debug`.
            for leaked in ["X-Amz-Signature", "top-secret"] {
                assert!(!text.contains(leaked), "{text}");
                assert!(!debug.contains(leaked), "{debug}");
            }
            // `Display` drops the host too. `Debug` keeps it, because hyper's
            // connect error prints the socket address; that is not a secret.
            assert!(!text.contains("127.0.0.1:1"), "{text}");
        }

        #[test]
        fn debug_does_not_leak_presigned_url() {
            let upload = test_upload("/s3/debug");
            let debug = format!("{upload:?}");
            assert!(!debug.contains(&upload.url), "{debug}");
            assert!(debug.contains("<redacted>"), "{debug}");
        }

        #[tokio::test]
        async fn create_and_upload_computes_checksum_and_fetches_artifact() {
            let path = temp_file(b"abc");
            let location = format!("{}/s3/create_and_upload", server_url());
            let _create = mock("POST", "/v1/artifacts")
                .match_body(Matcher::PartialJson(json!({
                    "data": { "attributes": {
                        "filesize": 3,
                        "checksum": "3a81oZNherrMQXNJriBBMRLm+k6JqX6iCp7u5ktV05ohkpkqJ0/BqDa6PCOj/uu9RU1EI2Q86A4qmslPpUyknw"
                    } }
                })))
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", &location)
                .with_body(artifact_body("artifact-cu", "WAITING"))
                .create();
            let put = mock("PUT", "/s3/create_and_upload")
                .match_body("abc")
                .with_status(200)
                .create();
            let _get = mock("GET", "/v1/artifacts/artifact-cu")
                .with_status(200)
                .with_header("content-type", "application/vnd.api+json")
                .with_body(artifact_body("artifact-cu", "UPLOADED"))
                .create();

            let mut request = create_request();
            request.filesize = None;
            let result =
                create_and_upload_with_client(&create_test_client(), request, &path, None).await;
            std::fs::remove_file(&path).ok();

            let artifact = result.expect("create and upload should succeed");
            put.assert();
            assert_eq!(artifact.status, ArtifactStatus::Uploaded);
        }

        #[tokio::test]
        async fn create_and_upload_names_artifact_when_status_fetch_fails() {
            let path = temp_file(b"hello");
            let location = format!("{}/s3/get_fails", server_url());
            let _create = mock("POST", "/v1/artifacts")
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", &location)
                .with_body(artifact_body("artifact-get-fail", "WAITING"))
                .create();
            let _put = mock("PUT", "/s3/get_fails")
                .match_body("hello")
                .with_status(200)
                .create();
            let _get = mock("GET", "/v1/artifacts/artifact-get-fail")
                .with_status(500)
                .create();

            let err =
                create_and_upload_with_client(&create_test_client(), create_request(), &path, None)
                    .await
                    .expect_err("status fetch should fail");
            std::fs::remove_file(&path).ok();

            let text = err.to_string();
            assert!(text.contains("artifact-get-fail"), "{text}");
            assert!(text.contains("was uploaded"), "{text}");
        }

        #[tokio::test]
        async fn create_and_upload_names_artifact_when_status_body_is_invalid() {
            let path = temp_file(b"hello");
            let location = format!("{}/s3/get_bad_body", server_url());
            let _create = mock("POST", "/v1/artifacts")
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", &location)
                .with_body(artifact_body("artifact-bad-body", "WAITING"))
                .create();
            let _put = mock("PUT", "/s3/get_bad_body")
                .match_body("hello")
                .with_status(200)
                .create();
            let _get = mock("GET", "/v1/artifacts/artifact-bad-body")
                .with_status(200)
                .with_header("content-type", "application/vnd.api+json")
                .with_body(r#"{"data": {"id": "artifact-bad-body"}}"#)
                .create();

            let err =
                create_and_upload_with_client(&create_test_client(), create_request(), &path, None)
                    .await
                    .expect_err("invalid status body should be an error");
            std::fs::remove_file(&path).ok();

            let text = err.to_string();
            assert!(text.contains("artifact-bad-body"), "{text}");
            assert!(text.contains("was uploaded"), "{text}");
        }

        #[tokio::test]
        async fn artifact_service_create_and_upload_uses_client() {
            let path = temp_file(b"hello");
            let location = format!("{}/s3/service_upload", server_url());
            let _create = mock("POST", "/v1/artifacts")
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", &location)
                .with_body(artifact_body("artifact-service", "WAITING"))
                .create();
            let put = mock("PUT", "/s3/service_upload")
                .match_body("hello")
                .with_status(200)
                .create();
            let _get = mock("GET", "/v1/artifacts/artifact-service")
                .with_status(200)
                .with_header("content-type", "application/vnd.api+json")
                .with_body(artifact_body("artifact-service", "UPLOADED"))
                .create();

            let artifact = create_test_keygen_client()
                .artifacts()
                .create_and_upload(create_request(), &path, None)
                .await
                .expect("service upload should succeed");
            std::fs::remove_file(&path).ok();

            put.assert();
            assert_eq!(artifact.id, "artifact-service");
            assert_eq!(artifact.status, ArtifactStatus::Uploaded);
        }

        #[tokio::test]
        async fn create_and_upload_deletes_artifact_when_upload_fails() {
            let path = temp_file(b"hello");
            let location = format!("{}/s3/create_and_upload_denied", server_url());
            let _create = mock("POST", "/v1/artifacts")
                .with_status(307)
                .with_header("content-type", "application/vnd.api+json")
                .with_header("location", &location)
                .with_body(artifact_body("artifact-denied", "WAITING"))
                .create();
            let _put = mock("PUT", "/s3/create_and_upload_denied")
                .with_status(403)
                .create();
            let delete = mock("DELETE", "/v1/artifacts/artifact-denied")
                .with_status(204)
                .create();

            let result =
                create_and_upload_with_client(&create_test_client(), create_request(), &path, None)
                    .await;
            std::fs::remove_file(&path).ok();

            let err = result.expect_err("failed upload should be an error");
            assert!(err.to_string().contains("403"));
            delete.assert();
        }

        #[tokio::test]
        async fn file_checksum_is_unpadded_base64_sha512() {
            let path = std::env::temp_dir().join(format!("keygen-rs-{}.bin", uuid::Uuid::new_v4()));
            tokio::fs::write(&path, b"abc").await.unwrap();

            let checksum = file_checksum(&path).await;
            tokio::fs::remove_file(&path).await.ok();

            assert_eq!(
                checksum.unwrap(),
                "3a81oZNherrMQXNJriBBMRLm+k6JqX6iCp7u5ktV05ohkpkqJ0/BqDa6PCOj/uu9RU1EI2Q86A4qmslPpUyknw"
            );
        }
    }
}
