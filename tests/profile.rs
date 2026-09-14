#![cfg(feature = "token")]

use keygen_rs::errors::Error;
use keygen_rs::profile::TokenPermissionConstraint;
use keygen_rs::KeygenClient;
use mockito::{mock, server_url};
use serde_json::{json, Value};

fn profile_document(permissions: Value) -> Value {
    json!({
        "data": {
            "id": "user-1",
            "type": "users",
            "attributes": {
                "email": "user@example.com"
            },
            "relationships": {}
        },
        "included": [
            {
                "id": "unrelated-1",
                "type": "products",
                "attributes": {},
                "relationships": {}
            },
            {
                "id": "token-for-another-user",
                "type": "tokens",
                "attributes": {
                    "kind": "user-token",
                    "expiry": null,
                    "permissions": ["*"],
                    "created": "2026-07-01T00:00:00.000Z",
                    "updated": "2026-07-02T00:00:00.000Z"
                },
                "relationships": {
                    "bearer": {
                        "data": {
                            "id": "user-2",
                            "type": "users"
                        }
                    }
                }
            },
            {
                "id": "token-1",
                "type": "tokens",
                "attributes": {
                    "kind": "user-token",
                    "expiry": "2026-08-01T00:00:00.000Z",
                    "permissions": permissions,
                    "created": "2026-07-01T00:00:00.000Z",
                    "updated": "2026-07-02T00:00:00.000Z"
                },
                "relationships": {
                    "bearer": {
                        "data": {
                            "id": "user-1",
                            "type": "users"
                        }
                    }
                }
            }
        ]
    })
}

fn test_client() -> KeygenClient {
    KeygenClient::builder()
        .account("account-1")
        .api_url(server_url())
        .environment("sandbox")
        .token("secret-token")
        .build()
        .expect("test client should build")
}

#[tokio::test]
async fn current_profile_returns_bearer_and_matching_included_token() {
    let _mock = mock("GET", "/v1/me")
        .match_header("authorization", "Bearer secret-token")
        .match_header("keygen-environment", "sandbox")
        .match_header("keygen-version", "1.8")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(profile_document(json!(["license.read", "license.update"])).to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile should parse");

    assert_eq!(profile.bearer().id(), "user-1");
    assert_eq!(profile.bearer().resource_type(), "users");
    assert_eq!(profile.token().id(), "token-1");
    assert_eq!(profile.token().kind(), "user-token");
    assert_eq!(profile.token().expiry(), Some("2026-08-01T00:00:00.000Z"));
    assert_eq!(
        profile
            .token()
            .permissions()
            .expect("permissions should be present"),
        ["license.read", "license.update"]
    );
}

#[tokio::test]
async fn current_token_exposes_constraint_not_effective_authorization() {
    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(profile_document(json!(["license.read", "*"])).to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile should parse");

    assert_eq!(
        profile.token().permission_constraint("license.read"),
        TokenPermissionConstraint::Explicit
    );
    assert_eq!(
        profile.token().permission_constraint("license.update"),
        TokenPermissionConstraint::Inherited
    );
}

#[tokio::test]
async fn current_token_reports_permissions_omitted_by_an_explicit_set() {
    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(profile_document(json!(["license.read"])).to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile should parse");

    assert_eq!(
        profile.token().permission_constraint("license.update"),
        TokenPermissionConstraint::Excluded
    );
}

#[tokio::test]
async fn current_profile_preserves_environment_token_kind() {
    let mut body = profile_document(json!(["*"]));
    body["included"][2]["attributes"]["kind"] = json!("environment-token");

    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(body.to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile should parse");

    assert_eq!(profile.token().kind(), "environment-token");
}

#[tokio::test]
async fn current_profile_preserves_future_token_kinds() {
    let mut body = profile_document(json!(["*"]));
    body["included"][2]["attributes"]["kind"] = json!("future-token");

    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(body.to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile should parse");

    assert_eq!(profile.token().kind(), "future-token");
}

#[tokio::test]
async fn current_token_with_an_empty_permission_set_excludes_every_permission() {
    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(profile_document(json!([])).to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile should parse");

    assert_eq!(profile.token().permissions(), Some(&[][..]));
    assert_eq!(
        profile.token().permission_constraint("license.read"),
        TokenPermissionConstraint::Excluded
    );
}

// Keygen CE does not serialize token permissions.
#[tokio::test]
async fn current_profile_does_not_treat_missing_permissions_as_empty() {
    let mut body = profile_document(json!([]));
    body["included"][2]["attributes"]
        .as_object_mut()
        .expect("token attributes should be an object")
        .remove("permissions");

    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(body.to_string())
        .create();

    let profile = test_client()
        .current_profile()
        .await
        .expect("profile without token permissions should parse");

    assert_eq!(profile.token().permissions(), None);
    assert_eq!(
        profile.token().permission_constraint("license.read"),
        TokenPermissionConstraint::Inherited
    );
}

#[tokio::test]
async fn current_profile_rejects_a_missing_or_mismatched_current_token() {
    let mut body = profile_document(json!(["license.read"]));
    body["included"][2]["relationships"]["bearer"]["data"]["id"] = json!("user-2");

    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(body.to_string())
        .create();

    let error = test_client()
        .current_profile()
        .await
        .expect_err("mismatched token should be rejected");

    assert!(matches!(error, Error::InvalidResponse(_)));
}

#[tokio::test]
async fn current_profile_rejects_multiple_matching_current_tokens() {
    let mut body = profile_document(json!(["license.read"]));
    let duplicate = body["included"][2].clone();
    body["included"]
        .as_array_mut()
        .expect("included should be an array")
        .push(duplicate);

    let _mock = mock("GET", "/v1/me")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(body.to_string())
        .create();

    let error = test_client()
        .current_profile()
        .await
        .expect_err("ambiguous token should be rejected");

    assert!(matches!(error, Error::InvalidResponse(_)));
}
