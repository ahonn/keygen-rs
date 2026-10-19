use keygen_rs::{release::ReleaseUpgradeRequest, KeygenClient};
use mockito::{mock, Matcher};
use serde_json::json;

fn client() -> KeygenClient {
    KeygenClient::builder()
        .account("account")
        .api_url(mockito::server_url())
        .product("config-product")
        .license_key("license-key")
        .build()
        .unwrap()
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
