use keygen_rs::{
    license::{LicenseCheckoutOpts, PaginationOptions},
    KeygenClient,
};
use mockito::{mock, Matcher};
use serde_json::json;

#[cfg(feature = "token")]
use keygen_rs::{license::LicenseTokenCreateRequest, token::TokenKind};

fn client() -> KeygenClient {
    let builder = KeygenClient::builder()
        .account("account")
        .api_url(mockito::server_url());
    #[cfg(feature = "token")]
    let builder = builder.token("admin-token");

    builder.build().unwrap()
}

fn license_response(id: &str) -> String {
    json!({
        "data": {
            "type": "licenses",
            "id": id,
            "attributes": {
                "key": "TEST-LICENSE-KEY",
                "metadata": {}
            },
            "relationships": {}
        }
    })
    .to_string()
}

#[cfg(feature = "token")]
fn user_response(id: &str) -> serde_json::Value {
    json!({
        "type": "users",
        "id": id,
        "attributes": {
            "email": "user@example.com",
            "firstName": "Test",
            "lastName": "User",
            "fullName": "Test User",
            "status": "ACTIVE",
            "role": "user",
            "permissions": [],
            "metadata": {},
            "lastSeenAt": null,
            "banReason": null,
            "created": "2026-01-01T00:00:00Z",
            "updated": "2026-01-02T00:00:00Z"
        },
        "relationships": {}
    })
}

fn entitlement_response(id: &str) -> serde_json::Value {
    json!({
        "type": "entitlements",
        "id": id,
        "attributes": {
            "name": "Feature",
            "code": "FEATURE",
            "metadata": {},
            "created": "2026-01-01T00:00:00Z",
            "updated": "2026-01-02T00:00:00Z"
        },
        "relationships": {
            "account": {
                "data": {
                    "type": "accounts",
                    "id": "account"
                }
            }
        }
    })
}

#[cfg(feature = "token")]
#[tokio::test]
async fn retrieves_a_license() {
    let api = mock("GET", "/v1/licenses/license-1")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client().licenses().get("license-1").await.unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn deletes_a_license() {
    let api = mock("DELETE", "/v1/licenses/license-1")
        .with_status(204)
        .create();

    client().licenses().delete("license-1").await.unwrap();

    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn suspends_a_license() {
    let api = mock("POST", "/v1/licenses/license-1/actions/suspend")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client().licenses().suspend("license-1").await.unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn reinstates_a_license() {
    let api = mock("POST", "/v1/licenses/license-1/actions/reinstate")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client().licenses().reinstate("license-1").await.unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn renews_a_license() {
    let api = mock("POST", "/v1/licenses/license-1/actions/renew")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client().licenses().renew("license-1").await.unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn revokes_a_license() {
    let api = mock("DELETE", "/v1/licenses/license-1/actions/revoke")
        .with_status(204)
        .create();

    client().licenses().revoke("license-1").await.unwrap();

    api.assert();
}

#[tokio::test]
async fn checks_out_a_license_as_json() {
    let api = mock("POST", "/v1/licenses/license-1/actions/check-out")
        .match_query(Matcher::UrlEncoded("ttl".into(), "3600".into()))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": {
                    "type": "license-files",
                    "id": "file-1",
                    "attributes": {
                        "certificate": "certificate-data",
                        "issued": "2026-01-01T00:00:00Z",
                        "expiry": "2026-01-01T01:00:00Z",
                        "ttl": 3600
                    },
                    "relationships": {}
                }
            })
            .to_string(),
        )
        .create();

    let file = client()
        .licenses()
        .checkout("license-1", &LicenseCheckoutOpts::with_ttl(3600))
        .await
        .unwrap();

    assert_eq!(file.id, "file-1");
    assert_eq!(file.certificate, "certificate-data");
    api.assert();
}

#[tokio::test]
async fn checks_out_a_license_as_plaintext_certificate() {
    let api = mock("GET", "/v1/licenses/license-1/actions/check-out")
        .match_query(Matcher::Any)
        .with_status(200)
        .with_header("content-type", "application/octet-stream")
        .with_body("KEYGEN-LICENSE-FILE")
        .create();

    let certificate = client()
        .licenses()
        .checkout_certificate("license-1", &LicenseCheckoutOpts::default())
        .await
        .unwrap();

    assert_eq!(certificate, "KEYGEN-LICENSE-FILE");
    api.assert();
}

#[tokio::test]
async fn checks_in_a_license() {
    let api = mock("POST", "/v1/licenses/license-1/actions/check-in")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client().licenses().check_in("license-1").await.unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn generates_a_license_token() {
    let api = mock("POST", "/v1/licenses/license-1/tokens")
        .match_body(Matcher::Json(json!({
            "data": {
                "type": "tokens",
                "attributes": {
                    "name": "Activation token",
                    "expiry": "2027-01-01T00:00:00Z",
                    "permissions": ["license.read", "license.validate"],
                    "maxActivations": 2,
                    "maxDeactivations": 1
                }
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": {
                    "type": "tokens",
                    "id": "token-1",
                    "attributes": {
                        "kind": "activation-token",
                        "token": "activ-token",
                        "name": "Activation token",
                        "expiry": "2027-01-01T00:00:00Z",
                        "permissions": ["license.read", "license.validate"],
                        "maxActivations": 2,
                        "activations": 0,
                        "maxDeactivations": 1,
                        "deactivations": 0,
                        "metadata": {},
                        "created": "2026-01-01T00:00:00Z",
                        "updated": "2026-01-01T00:00:00Z"
                    },
                    "relationships": {}
                }
            })
            .to_string(),
        )
        .create();
    let request = LicenseTokenCreateRequest {
        name: Some("Activation token".into()),
        expiry: Some("2027-01-01T00:00:00Z".into()),
        permissions: Some(vec!["license.read".into(), "license.validate".into()]),
        max_activations: Some(2),
        max_deactivations: Some(1),
    };

    let token = client()
        .licenses()
        .generate_token("license-1", Some(request))
        .await
        .unwrap();

    assert_eq!(token.id, "token-1");
    assert_eq!(token.kind, TokenKind::ActivationToken);
    assert_eq!(token.max_activations, Some(2));
    assert_eq!(token.max_deactivations, Some(1));
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn attaches_users_to_a_license() {
    let related = "/v1/licenses/license-1/users/user-1";
    let api = mock("POST", "/v1/licenses/license-1/users")
        .match_body(Matcher::Json(json!({
            "data": [
                {
                    "type": "users",
                    "id": "user-1"
                }
            ]
        })))
        .with_status(201)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": [
                    {
                        "type": "license-users",
                        "id": "license-user-1",
                        "attributes": {
                            "created": "2026-01-01T00:00:00Z",
                            "updated": "2026-01-02T00:00:00Z"
                        },
                        "relationships": {
                            "account": {
                                "data": { "type": "accounts", "id": "account" }
                            },
                            "license": {
                                "data": { "type": "licenses", "id": "license-1" }
                            },
                            "user": {
                                "data": { "type": "users", "id": "user-1" }
                            }
                        },
                        "links": {
                            "related": related
                        }
                    }
                ]
            })
            .to_string(),
        )
        .create();

    let attached = client()
        .licenses()
        .attach_users_with_response("license-1", &["user-1".into()])
        .await
        .unwrap();

    assert_eq!(attached.len(), 1);
    assert_eq!(attached[0].id, "license-user-1");
    assert_eq!(attached[0].account_id.as_deref(), Some("account"));
    assert_eq!(attached[0].license_id.as_deref(), Some("license-1"));
    assert_eq!(attached[0].user_id.as_deref(), Some("user-1"));
    assert_eq!(attached[0].related.as_deref(), Some(related));
    assert_eq!(
        attached[0].created.to_rfc3339(),
        "2026-01-01T00:00:00+00:00"
    );
    assert_eq!(
        attached[0].updated.to_rfc3339(),
        "2026-01-02T00:00:00+00:00"
    );
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn attaches_entitlements_to_a_license() {
    let related = "/v1/licenses/license-1/entitlements/entitlement-1";
    let api = mock("POST", "/v1/licenses/license-1/entitlements")
        .match_body(Matcher::Json(json!({
            "data": [
                {
                    "type": "entitlements",
                    "id": "entitlement-1"
                }
            ]
        })))
        .with_status(201)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": [
                    {
                        "type": "license-entitlements",
                        "id": "license-entitlement-1",
                        "attributes": {
                            "created": "2026-01-01T00:00:00Z",
                            "updated": "2026-01-02T00:00:00Z"
                        },
                        "relationships": {
                            "account": {
                                "data": { "type": "accounts", "id": "account" }
                            },
                            "license": {
                                "data": { "type": "licenses", "id": "license-1" }
                            },
                            "entitlement": {
                                "data": { "type": "entitlements", "id": "entitlement-1" }
                            }
                        },
                        "links": {
                            "related": related
                        }
                    }
                ]
            })
            .to_string(),
        )
        .create();

    let attached = client()
        .licenses()
        .attach_entitlements_with_response("license-1", &["entitlement-1".into()])
        .await
        .unwrap();

    assert_eq!(attached.len(), 1);
    assert_eq!(attached[0].id, "license-entitlement-1");
    assert_eq!(attached[0].account_id.as_deref(), Some("account"));
    assert_eq!(attached[0].license_id.as_deref(), Some("license-1"));
    assert_eq!(attached[0].entitlement_id.as_deref(), Some("entitlement-1"));
    assert_eq!(attached[0].related.as_deref(), Some(related));
    assert_eq!(
        attached[0].created.to_rfc3339(),
        "2026-01-01T00:00:00+00:00"
    );
    assert_eq!(
        attached[0].updated.to_rfc3339(),
        "2026-01-02T00:00:00+00:00"
    );
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn detaches_users_from_a_license() {
    let api = mock("DELETE", "/v1/licenses/license-1/users")
        .match_body(Matcher::Json(json!({
            "data": [
                {
                    "type": "users",
                    "id": "user-1"
                }
            ]
        })))
        .with_status(204)
        .create();

    client()
        .licenses()
        .detach_users("license-1", &["user-1".into()])
        .await
        .unwrap();

    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn lists_users_for_a_license_with_cursor_pagination() {
    let api = mock("GET", "/v1/licenses/license-1/users")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "25".into()),
            Matcher::UrlEncoded("page[size]".into(), "10".into()),
            Matcher::UrlEncoded("page[cursor]".into(), "cursor-1".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": [user_response("user-1")],
                "meta": {
                    "page": {
                        "cursor": "cursor-1",
                        "next": "cursor-2"
                    }
                },
                "links": {
                    "next": "/v1/licenses/license-1/users?page[cursor]=cursor-2"
                }
            })
            .to_string(),
        )
        .create();
    let options = PaginationOptions {
        limit: Some(25),
        page_size: Some(10),
        page_cursor: Some("cursor-1".into()),
    };

    let page = client()
        .licenses()
        .users("license-1", Some(&options))
        .await
        .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, "user-1");
    assert_eq!(page.meta.unwrap()["page"]["next"], "cursor-2");
    assert!(page.links.unwrap()["next"]
        .as_str()
        .unwrap()
        .contains("cursor-2"));
    api.assert();
}

#[tokio::test]
async fn lists_entitlements_for_a_license_with_cursor_pagination() {
    let api = mock("GET", "/v1/licenses/license-1/entitlements")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("limit".into(), "25".into()),
            Matcher::UrlEncoded("page[size]".into(), "10".into()),
            Matcher::UrlEncoded("page[cursor]".into(), "cursor-1".into()),
        ]))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(
            json!({
                "data": [entitlement_response("entitlement-1")],
                "meta": {
                    "page": {
                        "cursor": "cursor-1",
                        "next": "cursor-2"
                    }
                },
                "links": {
                    "next": "/v1/licenses/license-1/entitlements?page[cursor]=cursor-2"
                }
            })
            .to_string(),
        )
        .create();
    let options = PaginationOptions {
        limit: Some(25),
        page_size: Some(10),
        page_cursor: Some("cursor-1".into()),
    };

    let page = client()
        .licenses()
        .entitlements("license-1", Some(&options))
        .await
        .unwrap();

    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, "entitlement-1");
    assert_eq!(page.meta.unwrap()["page"]["next"], "cursor-2");
    assert!(page.links.unwrap()["next"]
        .as_str()
        .unwrap()
        .contains("cursor-2"));
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn changes_a_license_policy() {
    let api = mock("PUT", "/v1/licenses/license-1/policy")
        .match_body(Matcher::Json(json!({
            "data": {
                "type": "policies",
                "id": "policy-2"
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client()
        .licenses()
        .change_policy("license-1", "policy-2")
        .await
        .unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn changes_a_license_owner() {
    let api = mock("PUT", "/v1/licenses/license-1/owner")
        .match_body(Matcher::Json(json!({
            "data": {
                "type": "users",
                "id": "user-2"
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client()
        .licenses()
        .change_owner("license-1", "user-2")
        .await
        .unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}

#[cfg(feature = "token")]
#[tokio::test]
async fn changes_a_license_group() {
    let api = mock("PUT", "/v1/licenses/license-1/group")
        .match_body(Matcher::Json(json!({
            "data": {
                "type": "groups",
                "id": "group-2"
            }
        })))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(license_response("license-1"))
        .create();

    let license = client()
        .licenses()
        .change_group("license-1", "group-2")
        .await
        .unwrap();

    assert_eq!(license.id, "license-1");
    api.assert();
}
