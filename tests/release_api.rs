use keygen_rs::{
    errors::Error,
    release::{
        ArtifactDownloadOptions, Release, ReleaseChannel, ReleaseStatus, ReleaseUpgradeRequest,
    },
    KeygenClient,
};
use mockito::{mock, Matcher, Mock};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// Reproduce with calculateChecksum/calculateSignature from keygen-cli cmd/upload.go
// (commit 3fa3a56), curve25519-voi v0.0.0-20220328075252-7dd334e3daae:
// ed25519.NewKeyFromSeed(0x01..=0x20), SHA-512 of PAYLOAD, Ed25519ph with
// Context = CONTEXT, base64.RawStdEncoding. Same vectors as in src/artifact.rs tests.
const PAYLOAD: &[u8] = b"hello keygen";
const CONTEXT: &str = "test-product-id";
const PUBLIC_KEY: &str = "79b5562e8fe654f94078b112e8a98ba7901f853ae695bed7e0e3910bad049664";
const CHECKSUM: &str =
    "dnEi9Lf9kMb2ntwoiG5vSE5tXvJrGo0rLYTMBMcwLP5/TnwwbLgfzp8RuVIRTawUEiuwblnd6laUR0Zzt1BytA";
const SIGNATURE: &str =
    "YSpVi2Y333gwAyaSGZC67UDdslK8WjbH/QZ1DINDRMseUsNYd6ML++nPUgZqE+q3H1xaQ9y1OdegGH7jzFkPDg";

fn client() -> KeygenClient {
    KeygenClient::builder()
        .account("account")
        .api_url(mockito::server_url())
        .product("config-product")
        .license_key("license-key")
        .build()
        .unwrap()
}

fn release(id: &str) -> Release {
    Release {
        id: id.to_string(),
        name: None,
        description: None,
        version: "1.1.0".to_string(),
        semver: None,
        channel: ReleaseChannel::Stable,
        status: ReleaseStatus::Published,
        tag: None,
        metadata: None,
        created: "2024-01-01T00:00:00Z".to_string(),
        updated: "2024-01-01T00:00:00Z".to_string(),
        yanked_at: None,
        product_id: Some(CONTEXT.to_string()),
        package_id: None,
        account_id: None,
    }
}

/// Mocks the artifact redirect for `release_id` and the file behind it.
fn mock_artifact(
    release_id: &str,
    checksum: Option<&str>,
    signature: Option<&str>,
    payload: &[u8],
) -> (Mock, Mock) {
    let file_path = format!("/files/{release_id}/app");
    let redirect = mock(
        "GET",
        format!("/v1/releases/{release_id}/artifacts/app").as_str(),
    )
    .with_status(303)
    .with_header("content-type", "application/vnd.api+json")
    .with_header("location", &format!("{}{file_path}", mockito::server_url()))
    .with_body(
        json!({
            "data": {
                "id": "artifact-1",
                "type": "artifacts",
                "attributes": {
                    "filename": "app",
                    "filesize": payload.len(),
                    "checksum": checksum,
                    "signature": signature,
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
    let file = mock("GET", file_path.as_str())
        .match_header("authorization", Matcher::Missing)
        .with_status(200)
        .with_body(payload)
        .create();
    (redirect, file)
}

fn temp_path() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keygen-rs-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("app")
}

fn part_path(path: &Path) -> PathBuf {
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    PathBuf::from(part)
}

#[tokio::test]
async fn upgrades_from_a_version() {
    let api = mock("GET", "/v1/releases/1.0.0/upgrade")
        .match_header("authorization", "License license-key")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("product".into(), "config-product".into()),
            Matcher::UrlEncoded("channel".into(), "stable".into()),
            Matcher::UrlEncoded("constraint".into(), "1.0".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": {
                    "id": "release-2",
                    "type": "releases",
                    "attributes": {
                        "version": "1.1.0",
                        "channel": "stable",
                        "status": "PUBLISHED",
                        "created": "2024-01-01T00:00:00Z",
                        "updated": "2024-01-01T00:00:00Z"
                    },
                    "relationships": {}
                }
            })
            .to_string(),
        )
        .create();

    let request = ReleaseUpgradeRequest {
        constraint: Some("1.0".to_string()),
        ..Default::default()
    };
    let release = client()
        .upgrade_from("1.0.0", Some(&request))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(release.id, "release-2");
    assert_eq!(release.version, "1.1.0");
    api.assert();
}

#[tokio::test]
async fn returns_none_when_no_upgrade_is_available() {
    let api = mock("GET", "/v1/releases/1.1.0/upgrade")
        .match_query(Matcher::Any)
        .with_status(404)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "errors": [{
                    "title": "Not found",
                    "detail": "No upgrade found for release",
                    "code": "NOT_FOUND"
                }]
            })
            .to_string(),
        )
        .create();

    let release = client().upgrade_from("1.1.0", None).await.unwrap();

    assert!(release.is_none());
    api.assert();
}

#[tokio::test]
async fn downloads_and_verifies_a_signed_artifact() {
    let (redirect, file) = mock_artifact("dl-signed", Some(CHECKSUM), Some(SIGNATURE), PAYLOAD);
    let progress = Arc::new(Mutex::new(Vec::new()));
    let progress_sink = Arc::clone(&progress);
    let options = ArtifactDownloadOptions {
        public_key: Some(PUBLIC_KEY.to_string()),
        on_progress: Some(Box::new(move |done, total| {
            progress_sink.lock().unwrap().push((done, total));
        })),
    };
    let path = temp_path();

    let artifact = client()
        .download_artifact_to(&release("dl-signed"), "app", &path, Some(options))
        .await
        .unwrap();

    assert_eq!(artifact.id, "artifact-1");
    assert_eq!(std::fs::read(&path).unwrap(), PAYLOAD);
    assert!(!part_path(&path).exists());
    let size = PAYLOAD.len() as u64;
    assert_eq!(progress.lock().unwrap().last(), Some(&(size, Some(size))));
    redirect.assert();
    file.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn uses_the_configured_product_as_signature_context() {
    let (redirect, file) = mock_artifact("dl-context", Some(CHECKSUM), Some(SIGNATURE), PAYLOAD);
    let client = KeygenClient::builder()
        .account("account")
        .api_url(mockito::server_url())
        .product(CONTEXT)
        .license_key("license-key")
        .build()
        .unwrap();
    let options = ArtifactDownloadOptions {
        public_key: Some(PUBLIC_KEY.to_string()),
        ..Default::default()
    };
    let release = Release {
        product_id: None,
        ..release("dl-context")
    };
    let path = temp_path();

    client
        .download_artifact_to(&release, "app", &path, Some(options))
        .await
        .unwrap();

    redirect.assert();
    file.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn leaves_no_file_on_checksum_mismatch() {
    let (redirect, file) = mock_artifact("dl-tampered", Some(CHECKSUM), None, b"hello keygen!");
    let path = temp_path();

    let err = client()
        .download_artifact_to(&release("dl-tampered"), "app", &path, None)
        .await
        .unwrap_err();

    assert!(matches!(err, Error::ArtifactChecksumMismatch));
    assert!(!path.exists());
    assert!(!part_path(&path).exists());
    redirect.assert();
    file.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn requires_a_checksum() {
    let (redirect, file) = mock_artifact("dl-unverified", None, None, PAYLOAD);
    let file = file.expect(0);
    let path = temp_path();

    let err = client()
        .download_artifact_to(&release("dl-unverified"), "app", &path, None)
        .await
        .unwrap_err();

    assert!(matches!(err, Error::ArtifactChecksumMissing));
    redirect.assert();
    file.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn requires_a_signature_when_a_public_key_is_set() {
    let (redirect, file) = mock_artifact("dl-unsigned", Some(CHECKSUM), None, PAYLOAD);
    let file = file.expect(0);
    let options = ArtifactDownloadOptions {
        public_key: Some(PUBLIC_KEY.to_string()),
        ..Default::default()
    };
    let path = temp_path();

    let err = client()
        .download_artifact_to(&release("dl-unsigned"), "app", &path, Some(options))
        .await
        .unwrap_err();

    assert!(matches!(err, Error::ArtifactSignatureMissing));
    redirect.assert();
    file.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn rejects_the_account_public_key() {
    let redirect = mock("GET", "/v1/releases/dl-account-key/artifacts/app")
        .expect(0)
        .create();
    let client = KeygenClient::builder()
        .account("account")
        .api_url(mockito::server_url())
        .license_key("license-key")
        .public_key(PUBLIC_KEY.to_uppercase())
        .build()
        .unwrap();
    let options = ArtifactDownloadOptions {
        public_key: Some(PUBLIC_KEY.to_string()),
        ..Default::default()
    };
    let path = temp_path();

    let err = client
        .download_artifact_to(&release("dl-account-key"), "app", &path, Some(options))
        .await
        .unwrap_err();

    assert!(matches!(err, Error::ArtifactPublicKeyInvalid(_)));
    redirect.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}
