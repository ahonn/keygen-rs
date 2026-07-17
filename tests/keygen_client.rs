use keygen_rs::{ApiContractVersion, KeygenClient};
use mockito::{mock, server_url};

fn license_body(id: &str) -> String {
    format!(
        r#"{{"data":{{"type":"licenses","id":"{id}","attributes":{{"key":"TEST","metadata":{{}}}},"relationships":{{}}}}}}"#
    )
}

fn machine_body(id: &str) -> String {
    format!(
        r#"{{"data":{{"type":"machines","id":"{id}","attributes":{{"fingerprint":"fingerprint","name":null,"platform":"macos/aarch64","hostname":"host","ip":null,"cores":8,"metadata":{{}},"requireHeartbeat":true,"heartbeatStatus":"ALIVE","heartbeatDuration":300,"created":"2026-01-01T00:00:00Z","updated":"2026-01-01T00:00:00Z"}},"relationships":{{}}}}}}"#
    )
}

#[tokio::test]
async fn clients_keep_api_contract_versions_isolated() {
    let v1_7_mock = mock("POST", "/v1/licenses/lic-17/actions/increment-usage")
        .match_header("Keygen-Version", "1.7")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(license_body("lic-17"))
        .create();
    let v1_8_mock = mock("POST", "/v1/licenses/lic-18/actions/increment-usage")
        .match_header("Keygen-Version", "1.8")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(license_body("lic-18"))
        .create();

    let v1_7 = KeygenClient::builder()
        .account("account")
        .api_url(server_url())
        .api_contract_version(ApiContractVersion::V1_7)
        .verify_keygen_signature(false)
        .build()
        .unwrap();
    let v1_8 = KeygenClient::builder()
        .account("account")
        .api_url(server_url())
        .api_contract_version(ApiContractVersion::V1_8)
        .verify_keygen_signature(false)
        .build()
        .unwrap();

    let license_17 = v1_7
        .licenses()
        .increment_usage("lic-17", None)
        .await
        .unwrap();
    let license_18 = v1_8
        .licenses()
        .increment_usage("lic-18", None)
        .await
        .unwrap();

    assert_eq!(license_17.id, "lic-17");
    assert_eq!(license_18.id, "lic-18");
    v1_7_mock.assert();
    v1_8_mock.assert();
}

#[test]
fn client_defaults_to_the_current_api_contract() {
    let client = KeygenClient::builder().build().unwrap();

    assert_eq!(client.api_contract_version(), ApiContractVersion::CURRENT);
}

#[tokio::test]
async fn client_service_info_observes_the_account_contract_without_overriding_it() {
    let service_mock = mock("GET", "/v1/ping")
        .match_header("Keygen-Version", mockito::Matcher::Missing)
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_header("Keygen-Version", "1.8")
        .with_body("ok")
        .create();
    let client = KeygenClient::builder()
        .account("account")
        .api_url(server_url())
        .api_contract_version(ApiContractVersion::V1_7)
        .build()
        .unwrap();

    let info = client.service_info().await.unwrap();

    assert_eq!(info.api_version.as_deref(), Some("1.8"));
    service_mock.assert();
}

#[tokio::test]
async fn activated_machine_keeps_the_originating_api_contract() {
    let activate_mock = mock("POST", "/v1/machines")
        .match_header("Keygen-Version", "1.7")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(machine_body("machine-17"))
        .create();
    let ping_mock = mock("POST", "/v1/machines/machine-17/actions/ping")
        .match_header("Keygen-Version", "1.7")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(machine_body("machine-17"))
        .create();
    let client = KeygenClient::builder()
        .account("account")
        .api_url(server_url())
        .api_contract_version(ApiContractVersion::V1_7)
        .verify_keygen_signature(false)
        .build()
        .unwrap();

    let machine = client
        .licenses()
        .activate("license-17", "fingerprint", &[])
        .await
        .unwrap();
    let pinged = machine.ping().await.unwrap();

    assert_eq!(pinged.id, "machine-17");
    activate_mock.assert();
    ping_mock.assert();
}
