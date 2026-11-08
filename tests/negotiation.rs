#![cfg(feature = "token")]
//! `KeygenClient::negotiate` against stand-ins for the kinds of Keygen server.
//! Each test sends its own User-Agent, so its mocks answer only its requests.

use keygen_rs::license::LicenseListOptions;
use keygen_rs::{ApiContractVersion, KeygenClient, Pagination};
use mockito::{mock, server_url, Matcher};

fn client(user_agent: &str, version: ApiContractVersion) -> KeygenClient {
    KeygenClient::builder()
        .api_url(server_url())
        .account("test-account")
        .token("admin-token")
        .api_contract_version(version)
        .user_agent(user_agent)
        .build()
        .unwrap()
}

fn ping(user_agent: &str, version: &str) -> mockito::Mock {
    mock("GET", "/v1/ping")
        .match_header("user-agent", user_agent)
        .match_header("keygen-version", Matcher::Missing)
        .with_status(200)
        .with_header("keygen-edition", "CE")
        .with_header("keygen-version", version)
        .create()
}

fn cursor_probe(user_agent: &str, version: &str) -> mockito::Mock {
    mock("GET", "/v1/licenses")
        .match_header("user-agent", user_agent)
        .match_header("keygen-version", version)
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("page[size]".into(), "1".into()),
            Matcher::UrlEncoded("page[cursor]".into(), String::new()),
        ]))
}

/// Keygen CE before 1.7.0 serves API 1.7 and pages with page numbers only.
#[tokio::test]
async fn an_old_keygen_ce_server_gets_api_1_7_and_page_numbers() {
    let ua = "negotiation-old-ce";
    let _ping = ping(ua, "1.7");
    let _probe = cursor_probe(ua, "1.7")
        .with_status(400)
        .with_body(
            r#"{"errors":[{"title":"Bad request","detail":"page number must be a number greater than 0 (got 0)","code":"PARAMETER_INVALID","source":{"parameter":"page"}}]}"#,
        )
        .create();

    let client = client(ua, ApiContractVersion::V1_8)
        .negotiate()
        .await
        .unwrap();

    let profile = client.server_profile().unwrap();
    assert_eq!(profile.api_version, ApiContractVersion::V1_7);
    assert_eq!(profile.pagination, Pagination::PageNumber);
    assert_eq!(client.api_contract_version(), ApiContractVersion::V1_7);

    // The first page: no cursor, page 1.
    let first = mock("GET", "/v1/licenses")
        .match_header("user-agent", ua)
        .match_header("keygen-version", "1.7")
        .match_query(Matcher::Exact("page%5Bnumber%5D=1&page%5Bsize%5D=2".into()))
        .with_status(200)
        .with_body(
            r#"{"data":[],"links":{"next":"/v1/licenses?page%5Bnumber%5D=2&page%5Bsize%5D=2"}}"#,
        )
        .create();
    let options = LicenseListOptions {
        page_size: Some(2),
        ..Default::default()
    };
    let page = client.licenses().list(Some(&options)).await.unwrap();
    first.assert();
    let next = page.next_cursor();
    assert_eq!(next.as_deref(), Some("2"));

    // The next page: the token from `next_cursor` is the page number.
    let second = mock("GET", "/v1/licenses")
        .match_header("user-agent", ua)
        .match_query(Matcher::Exact("page%5Bnumber%5D=2&page%5Bsize%5D=2".into()))
        .with_status(200)
        .with_body(r#"{"data":[],"links":{}}"#)
        .create();
    let options = LicenseListOptions {
        page_size: Some(2),
        page_cursor: next,
        ..Default::default()
    };
    let page = client.licenses().list(Some(&options)).await.unwrap();
    second.assert();
    assert_eq!(page.next_cursor(), None);
}

/// Keygen Cloud and Keygen CE 1.7.0 and later page with cursors.
#[tokio::test]
async fn a_cursor_server_keeps_the_current_version_and_cursors() {
    let ua = "negotiation-cloud";
    let _ping = ping(ua, "1.8");
    let probe = cursor_probe(ua, "1.8")
        .with_status(200)
        .with_body(r#"{"data":[],"links":{}}"#)
        .create();

    let client = client(ua, ApiContractVersion::V1_8)
        .negotiate()
        .await
        .unwrap();

    probe.assert();
    let profile = client.server_profile().unwrap();
    assert_eq!(profile.api_version, ApiContractVersion::V1_8);
    assert_eq!(profile.pagination, Pagination::Cursor);
    assert_eq!(profile.capabilities.api_version.as_deref(), Some("1.8"));
}

/// A client that asks for an older version than the server's keeps its own.
#[tokio::test]
async fn a_client_keeps_its_older_version() {
    let ua = "negotiation-older-client";
    let _ping = ping(ua, "1.8");
    let _probe = cursor_probe(ua, "1.7")
        .with_status(200)
        .with_body(r#"{"data":[]}"#)
        .create();

    let client = client(ua, ApiContractVersion::V1_7)
        .negotiate()
        .await
        .unwrap();

    assert_eq!(
        client.server_profile().unwrap().api_version,
        ApiContractVersion::V1_7
    );
}

/// Refused credentials fail the negotiation with the server's answer.
#[tokio::test]
async fn refused_credentials_fail_the_negotiation() {
    let ua = "negotiation-unauthorized";
    let _ping = ping(ua, "1.8");
    let _probe = cursor_probe(ua, "1.8")
        .with_status(401)
        .with_body(r#"{"errors":[{"title":"Unauthorized","detail":"You must be authenticated","code":"TOKEN_INVALID"}]}"#)
        .create();

    let error = client(ua, ApiContractVersion::V1_8)
        .negotiate()
        .await
        .unwrap_err();

    let status = error.api_error().map(|document| document.status().as_u16());
    assert_eq!(status, Some(401));
}
