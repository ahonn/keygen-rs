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
use std::time::Duration;

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
    let redirect = mock_artifact_redirect(
        release_id,
        checksum,
        signature,
        payload.len(),
        &format!("{}{file_path}", mockito::server_url()),
    );
    let file = mock("GET", file_path.as_str())
        .match_header("authorization", Matcher::Missing)
        .with_status(200)
        .with_body(payload)
        .create();
    (redirect, file)
}

fn mock_artifact_redirect(
    release_id: &str,
    checksum: Option<&str>,
    signature: Option<&str>,
    filesize: usize,
    location: &str,
) -> Mock {
    mock(
        "GET",
        format!("/v1/releases/{release_id}/artifacts/app").as_str(),
    )
    .with_status(303)
    .with_header("content-type", "application/vnd.api+json")
    .with_header("location", location)
    .with_body(
        json!({
            "data": {
                "id": "artifact-1",
                "type": "artifacts",
                "attributes": {
                    "filename": "app",
                    "filesize": filesize,
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
    .create()
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

fn assert_directory_contents(directory: &Path, expected: &[&str]) {
    let mut actual = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<_>>();
    actual.sort();
    let mut expected = expected.to_vec();
    expected.sort();
    assert_eq!(actual, expected);
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
    assert_directory_contents(path.parent().unwrap(), &["app"]);
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
    assert_directory_contents(path.parent().unwrap(), &[]);
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
    assert_directory_contents(path.parent().unwrap(), &[]);
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
    assert_directory_contents(path.parent().unwrap(), &[]);
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
    assert_directory_contents(path.parent().unwrap(), &[]);
    redirect.assert();
    std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
}

#[tokio::test]
async fn checksum_failure_preserves_the_existing_destination() {
    let (redirect, file) = mock_artifact("dl-preserve", Some(CHECKSUM), None, b"invalid update");
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app");
    std::fs::write(&path, b"installed version").unwrap();

    let result = client()
        .download_artifact_to(&release("dl-preserve"), "app", &path, None)
        .await;

    assert!(matches!(result, Err(Error::ArtifactChecksumMismatch)));
    assert_eq!(std::fs::read(&path).unwrap(), b"installed version");
    assert_directory_contents(directory.path(), &["app"]);
    redirect.assert();
    file.assert();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_downloads_keep_their_verified_bytes_and_cleanup_isolated() {
    let (redirect_a, file_a) = mock_artifact("dl-concurrent-a", Some(CHECKSUM), None, PAYLOAD);
    let (redirect_b, file_b) =
        mock_artifact("dl-concurrent-b", Some(CHECKSUM), None, b"invalid update");
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app");
    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let (resume_tx, resume_rx) = std::sync::mpsc::channel();
    let mut ready_tx = Some(ready_tx);
    let options = ArtifactDownloadOptions {
        on_progress: Some(Box::new(move |_, _| {
            if let Some(ready_tx) = ready_tx.take() {
                ready_tx.send(()).unwrap();
                tokio::task::block_in_place(|| {
                    resume_rx.recv_timeout(Duration::from_secs(10)).unwrap();
                });
            }
        })),
        ..Default::default()
    };
    let path_a = path.clone();
    let download_a = tokio::spawn(async move {
        client()
            .download_artifact_to(&release("dl-concurrent-a"), "app", &path_a, Some(options))
            .await
    });
    tokio::time::timeout(Duration::from_secs(10), ready_rx)
        .await
        .unwrap()
        .unwrap();

    // B fails while A is paused. B must not overwrite or remove A's temporary file.
    let result_b = client()
        .download_artifact_to(&release("dl-concurrent-b"), "app", &path, None)
        .await;
    resume_tx.send(()).unwrap();
    let result_a = download_a.await.unwrap();

    assert!(matches!(result_b, Err(Error::ArtifactChecksumMismatch)));
    result_a.unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), PAYLOAD);
    assert_directory_contents(directory.path(), &["app"]);
    redirect_a.assert();
    file_a.assert();
    redirect_b.assert();
    file_b.assert();
}

#[tokio::test]
async fn leaves_an_existing_predictable_part_file_untouched() {
    let (redirect, file) = mock_artifact("dl-existing-part", Some(CHECKSUM), None, PAYLOAD);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app");
    std::fs::write(part_path(&path), b"another download").unwrap();

    client()
        .download_artifact_to(&release("dl-existing-part"), "app", &path, None)
        .await
        .unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), PAYLOAD);
    assert_eq!(
        std::fs::read(part_path(&path)).unwrap(),
        b"another download"
    );
    assert_directory_contents(directory.path(), &["app", "app.part"]);
    redirect.assert();
    file.assert();
}

#[cfg(unix)]
#[tokio::test]
async fn leaves_an_existing_predictable_part_symlink_untouched() {
    let (redirect, file) = mock_artifact("dl-part-symlink", Some(CHECKSUM), None, PAYLOAD);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app");
    let target = directory.path().join("user-file");
    std::fs::write(&target, b"valuable existing data").unwrap();
    std::os::unix::fs::symlink(&target, part_path(&path)).unwrap();

    client()
        .download_artifact_to(&release("dl-part-symlink"), "app", &path, None)
        .await
        .unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), PAYLOAD);
    assert_eq!(std::fs::read(&target).unwrap(), b"valuable existing data");
    assert_eq!(std::fs::read_link(part_path(&path)).unwrap(), target);
    assert_directory_contents(directory.path(), &["app", "app.part", "user-file"]);
    redirect.assert();
    file.assert();
}

#[tokio::test]
async fn cancellation_removes_a_partial_download() {
    use std::io::{Read, Write};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let location = format!("http://{}/app", listener.local_addr().unwrap());
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        // Advertise one additional byte so the download waits after reporting progress.
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            PAYLOAD.len() + 1
        )
        .unwrap();
        stream.write_all(PAYLOAD).unwrap();
        stream.flush().unwrap();
        let _ = stop_rx.recv_timeout(Duration::from_secs(10));
    });
    let redirect = mock_artifact_redirect(
        "dl-cancel",
        Some(CHECKSUM),
        None,
        PAYLOAD.len() + 1,
        &location,
    );
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app");
    let (progress_tx, progress_rx) = tokio::sync::oneshot::channel();
    let mut progress_tx = Some(progress_tx);
    let options = ArtifactDownloadOptions {
        on_progress: Some(Box::new(move |_, _| {
            if let Some(progress_tx) = progress_tx.take() {
                let _ = progress_tx.send(());
            }
        })),
        ..Default::default()
    };
    let download = tokio::spawn(async move {
        client()
            .download_artifact_to(&release("dl-cancel"), "app", &path, Some(options))
            .await
    });
    tokio::time::timeout(Duration::from_secs(10), progress_rx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);

    download.abort();
    assert!(download.await.unwrap_err().is_cancelled());
    stop_tx.send(()).unwrap();
    server.join().unwrap();

    assert_directory_contents(directory.path(), &[]);
    redirect.assert();
}

#[tokio::test]
async fn failed_publish_removes_the_temporary_file() {
    let (redirect, file) = mock_artifact("dl-rename-failure", Some(CHECKSUM), None, PAYLOAD);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("app");
    std::fs::create_dir(&path).unwrap();
    std::fs::write(path.join("existing"), b"keep this file").unwrap();

    let result = client()
        .download_artifact_to(&release("dl-rename-failure"), "app", &path, None)
        .await;

    assert!(matches!(result, Err(Error::UnexpectedError(_))));
    assert_eq!(
        std::fs::read(path.join("existing")).unwrap(),
        b"keep this file"
    );
    assert_directory_contents(directory.path(), &["app"]);
    redirect.assert();
    file.assert();
}

#[tokio::test]
async fn downloads_to_a_bare_relative_filename() {
    let (redirect, file) = mock_artifact("dl-relative", Some(CHECKSUM), None, PAYLOAD);
    // Keep a cleanup guard without changing the process-wide working directory.
    let destination = tempfile::NamedTempFile::new_in(".").unwrap();
    let path = Path::new(destination.path().file_name().unwrap());

    client()
        .download_artifact_to(&release("dl-relative"), "app", path, None)
        .await
        .unwrap();

    assert_eq!(std::fs::read(path).unwrap(), PAYLOAD);
    redirect.assert();
    file.assert();
}
