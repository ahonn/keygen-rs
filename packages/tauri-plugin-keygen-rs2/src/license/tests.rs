use super::*;
use keygen_rs::config::KeygenConfig;
use mockito::{mock, Matcher};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{
    test::{mock_builder, mock_context, noop_assets, MockRuntime},
    Manager,
};

struct TestApp {
    app: tauri::App<MockRuntime>,
    directory: PathBuf,
}

impl TestApp {
    fn new(case: &str, fixtures: &Value) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let mut context = mock_context(noop_assets());
        context.config_mut().identifier = format!(
            "dev.keygen.cache-test-{}-{}-{case}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        );
        let app = mock_builder().build(context).unwrap();
        app.manage(tokio::sync::Mutex::new(KeygenConfig {
            account: case.to_string(),
            api_url: mockito::server_url(),
            api_prefix: case.to_string(),
            license_key: Some(fixtures["key"].as_str().unwrap().to_string()),
            public_key: Some(fixtures["publicKey"].as_str().unwrap().to_string()),
            // HTTP responses are mocked; certificate verification remains enabled.
            verify_keygen_signature: Some(false),
            ..Default::default()
        }));
        let directory = app.path().app_data_dir().unwrap();
        Self { app, directory }
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if self.directory.exists() {
            fs::remove_dir_all(&self.directory).unwrap();
        }
    }
}

fn fixtures() -> Value {
    serde_json::from_str(include_str!("../../tests/fixtures/license-files.json")).unwrap()
}

fn file(fixtures: &Value, name: &str) -> LicenseFile {
    serde_json::from_value(fixtures["files"][name].clone()).unwrap()
}

fn license(key: &str) -> License {
    let mut license = License::from_id("cache-test-license");
    license.key = key.to_string();
    license
}

fn checkout_mock(case: &str, file: &LicenseFile, encrypted: bool) -> mockito::Mock {
    mock(
        "POST",
        format!("/{case}/licenses/cache-test-license/actions/check-out").as_str(),
    )
    .match_query(Matcher::AllOf(vec![
        Matcher::UrlEncoded("include".into(), "entitlements".into()),
        Matcher::UrlEncoded("encrypt".into(), encrypted.to_string()),
    ]))
    .with_status(200)
    .with_header("content-type", "application/vnd.api+json")
    .with_body(
        json!({
            "data": {
                "type": "license-files", "id": file.id, "relationships": {},
                "attributes": {
                    "certificate": file.certificate,
                    "issued": file.issued, "expiry": file.expiry, "ttl": file.ttl
                }
            }
        })
        .to_string(),
    )
    .create()
}

#[tokio::test]
async fn checkout_preserves_entitlements_after_cache_reload_for_both_encodings() {
    let fixtures = fixtures();
    let key = fixtures["key"].as_str().unwrap();
    let public_key = fixtures["publicKey"].as_str().unwrap();
    for (name, encrypted) in [("plain", false), ("encrypted", true)] {
        let case = format!("roundtrip-{name}");
        let file = file(&fixtures, name);
        let response = checkout_mock(&case, &file, encrypted);
        let app = TestApp::new(&case, &fixtures);
        let handle = app.app.handle();
        let license = license(key);
        LicenseState::save_license_key_cache(handle, &license).unwrap();
        let mut state = LicenseState {
            valid: true,
            license: Some(license),
            ..Default::default()
        };
        let options = if encrypted {
            LicenseCheckoutOpts::new()
        } else {
            LicenseCheckoutOpts::new().with_encrypt(false)
        };

        state.checkout(handle, &options).await.unwrap();
        response.assert();
        assert_eq!(state.included.unwrap().entitlements[0].code, "PRO");
        assert_eq!(
            LicenseState::read_license_file(handle).unwrap().unwrap(),
            file.certificate
        );
        let restored = LicenseState::load(handle, public_key).unwrap();
        assert!(restored.valid);
        assert_eq!(restored.key.as_deref(), Some(key));
        assert_eq!(restored.license.unwrap().id, "cache-test-license");
        assert_eq!(restored.included.unwrap().entitlements[0].code, "PRO");
    }
}

#[tokio::test]
async fn checkout_uses_certificate_key_override_instead_of_http_verification_key() {
    let fixtures = fixtures();
    for (name, encrypted) in [("plain", false), ("encrypted", true)] {
        let case = format!("key-override-{name}");
        let file = file(&fixtures, name);
        let response = checkout_mock(&case, &file, encrypted);
        let app = TestApp::new(&case, &fixtures);
        let handle = app.app.handle();
        handle.get_keygen_config().lock().await.public_key = Some("00".repeat(32));
        handle.manage(crate::LicenseFilePublicKey(
            fixtures["publicKey"].as_str().unwrap().to_string(),
        ));
        let mut state = LicenseState {
            license: Some(license(fixtures["key"].as_str().unwrap())),
            ..Default::default()
        };

        state
            .checkout(handle, &LicenseCheckoutOpts::new().with_encrypt(encrypted))
            .await
            .unwrap();
        response.assert();
        assert_eq!(state.included.unwrap().entitlements[0].code, "PRO");
        assert_eq!(
            LicenseState::read_license_file(handle).unwrap().unwrap(),
            file.certificate
        );
    }
}

#[tokio::test]
async fn checkout_rejects_tampered_certificates_before_replacing_cache() {
    let fixtures = fixtures();
    for (name, encrypted) in [("plain", false), ("encrypted", true)] {
        let case = format!("tampered-checkout-{name}");
        let original = file(&fixtures, name);
        let tampered = file(&fixtures, &format!("{name}-tampered"));
        let response = checkout_mock(&case, &tampered, encrypted);
        let app = TestApp::new(&case, &fixtures);
        let handle = app.app.handle();
        LicenseState::save_license_file(handle, &original).unwrap();
        let mut state = LicenseState {
            license: Some(license(fixtures["key"].as_str().unwrap())),
            ..Default::default()
        };
        let result = state
            .checkout(handle, &LicenseCheckoutOpts::new().with_encrypt(encrypted))
            .await;
        response.assert();
        assert!(matches!(
            result,
            Err(Error::KeygenError(KeygenError::LicenseFileNotGenuine(_)))
        ));
        assert!(state.included.is_none());
        assert_eq!(
            LicenseState::read_license_file(handle).unwrap().unwrap(),
            original.certificate
        );
    }
}

#[test]
fn startup_rejects_cache_for_another_license_key() {
    let fixtures = fixtures();
    let public_key = fixtures["publicKey"].as_str().unwrap();
    for name in ["plain", "encrypted"] {
        let app = TestApp::new(&format!("wrong-key-{name}"), &fixtures);
        let handle = app.app.handle();
        LicenseState::save_license_key_cache(handle, &license("OTHER-LICENSE-KEY")).unwrap();
        LicenseState::save_license_file(handle, &file(&fixtures, name)).unwrap();
        match LicenseState::load(handle, public_key) {
            Err(Error::KeygenError(KeygenError::LicenseFileInvalid(_))) if name == "plain" => (),
            Err(Error::KeygenError(KeygenError::DecryptionError(_))) if name == "encrypted" => (),
            result => panic!("Expected mismatched cache to be rejected: {result:?}"),
        }
    }
}

#[test]
fn startup_exposes_expired_data_only_after_signature_verification() {
    let fixtures = fixtures();
    let public_key = fixtures["publicKey"].as_str().unwrap();
    let key = fixtures["key"].as_str().unwrap();
    for name in ["plain", "encrypted"] {
        let app = TestApp::new(&format!("expired-{name}"), &fixtures);
        let handle = app.app.handle();
        LicenseState::save_license_key_cache(handle, &license(key)).unwrap();
        LicenseState::save_license_file(handle, &file(&fixtures, &format!("{name}-expired")))
            .unwrap();
        match LicenseState::load(handle, public_key) {
            Err(Error::KeygenError(KeygenError::LicenseFileExpired(dataset))) => {
                assert_eq!(dataset.license.key, key);
                assert_eq!(dataset.offline_entitlements().unwrap()[0].code, "PRO");
            }
            result => panic!("Expected verified expired data: {result:?}"),
        }
        LicenseState::save_license_file(
            handle,
            &file(&fixtures, &format!("{name}-expired-tampered")),
        )
        .unwrap();
        assert!(matches!(
            LicenseState::load(handle, public_key),
            Err(Error::KeygenError(KeygenError::LicenseFileNotGenuine(_)))
        ));
    }
}
