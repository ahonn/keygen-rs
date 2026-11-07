//! Processes, second factors, event logs, request logs, search, capabilities and
//! rate limit status through a [`KeygenClient`].
#![cfg(feature = "token")]

use keygen_rs::errors::Error;
use keygen_rs::event_log::ListEventLogsOptions;
use keygen_rs::pagination::PaginationOptions;
use keygen_rs::process::{
    ListProcessesOptions, ProcessStatus, SpawnProcessRequest, UpdateProcessRequest,
};
use keygen_rs::request_log::ListRequestLogsOptions;
use keygen_rs::second_factor::SecondFactorCredential;
use keygen_rs::{KeygenClient, KeygenEdition, KeygenMode, ResourceIdentifier};
use mockito::{mock, Matcher, Mock};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

const TOKEN: &str = "admin-token";
const ENVIRONMENT: &str = "admin-env";
const TIMESTAMP: &str = "2026-01-01T00:00:00Z";

fn client() -> KeygenClient {
    client_with_environment(ENVIRONMENT)
}

fn client_with_environment(environment: &str) -> KeygenClient {
    KeygenClient::builder()
        .account("account")
        .api_url(mockito::server_url())
        .token(TOKEN)
        .environment(environment)
        .build()
        .unwrap()
}

fn linkage(kind: &str, id: &str) -> Value {
    json!({ "data": { "type": kind, "id": id } })
}

fn resource(kind: &str, id: &str, mut attributes: Value, relationships: Value) -> Value {
    attributes["created"] = json!(TIMESTAMP);
    attributes["updated"] = json!(TIMESTAMP);
    json!({ "type": kind, "id": id, "attributes": attributes, "relationships": relationships })
}

fn single(data: Value) -> String {
    json!({ "data": data }).to_string()
}

fn list(data: Vec<Value>, next: Option<&str>) -> String {
    json!({ "data": data, "meta": {}, "links": { "self": "/v1/x", "next": next } }).to_string()
}

fn query(pairs: &[(&str, &str)]) -> Matcher {
    Matcher::AllOf(
        pairs
            .iter()
            .map(|(key, value)| Matcher::UrlEncoded(key.to_string(), value.to_string()))
            .collect(),
    )
}

/// A mock that only answers requests carrying this file's token and environment.
fn service_mock(
    method: &str,
    path: &str,
    query: Matcher,
    status: usize,
    body: Option<String>,
) -> Mock {
    let mock = mock(method, path)
        .match_query(query)
        .match_header("authorization", format!("Bearer {TOKEN}").as_str())
        .match_header("Keygen-Environment", ENVIRONMENT)
        .with_status(status)
        .expect(1);
    match body {
        Some(body) => mock
            .with_header("content-type", "application/vnd.api+json")
            .with_body(body),
        None => mock,
    }
    .create()
}

fn process(id: &str) -> Value {
    resource(
        "processes",
        id,
        json!({
            "pid": "4242",
            "status": "ALIVE",
            "lastHeartbeat": TIMESTAMP,
            "nextHeartbeat": "2026-01-01T00:10:00Z",
            "interval": 600,
            "metadata": { "host": "build-01" }
        }),
        json!({
            "account": linkage("accounts", "account"),
            "environment": { "data": null },
            "product": linkage("products", "prod-1"),
            "license": linkage("licenses", "lic-1"),
            "machine": linkage("machines", "mach-1")
        }),
    )
}

#[tokio::test]
async fn process_list_sends_filters_and_follows_the_cursor() {
    let first = service_mock(
        "GET",
        "/v1/processes",
        query(&[
            ("machine", "mach-1"),
            ("license", "lic-1"),
            ("owner", "user-1"),
            ("user", "user-2"),
            ("product", "prod-1"),
            ("page[size]", "1"),
            ("page[cursor]", ""),
        ]),
        200,
        Some(list(
            vec![process("proc-1")],
            Some("/v1/processes?page%5Bsize%5D=1&page%5Bcursor%5D=proc-1"),
        )),
    );
    let last = service_mock(
        "GET",
        "/v1/processes",
        query(&[("page[size]", "1"), ("page[cursor]", "proc-1")]),
        200,
        Some(list(vec![process("proc-2")], None)),
    );
    let client = client();

    let page = client
        .processes()
        .list(Some(&ListProcessesOptions {
            page_size: Some(1),
            machine: Some("mach-1".into()),
            license: Some("lic-1".into()),
            owner: Some("user-1".into()),
            user: Some("user-2".into()),
            product: Some("prod-1".into()),
            ..Default::default()
        }))
        .await
        .unwrap();
    let process = &page.items()[0];
    assert_eq!(process.id, "proc-1");
    assert_eq!(process.pid, "4242");
    assert_eq!(process.status, ProcessStatus::Alive);
    assert_eq!(process.interval, Some(600));
    assert_eq!(process.machine_id.as_deref(), Some("mach-1"));
    assert_eq!(process.license_id.as_deref(), Some("lic-1"));
    assert_eq!(process.product_id.as_deref(), Some("prod-1"));
    assert_eq!(process.environment_id, None);
    let cursor = page.next_cursor().expect("first page has a next cursor");

    let page = client
        .processes()
        .list(Some(&ListProcessesOptions {
            page_size: Some(1),
            page_cursor: Some(cursor),
            ..Default::default()
        }))
        .await
        .unwrap();
    assert_eq!(page.items()[0].id, "proc-2");
    assert!(!page.has_more());
    first.assert();
    last.assert();
}

#[tokio::test]
async fn process_service_gets_spawns_pings_updates_and_kills() {
    let get = service_mock(
        "GET",
        "/v1/processes/proc-get",
        Matcher::Missing,
        200,
        Some(single(process("proc-get"))),
    );
    let spawn = mock("POST", "/v1/processes")
        .match_header("authorization", format!("Bearer {TOKEN}").as_str())
        .match_body(Matcher::Json(json!({
            "data": {
                "type": "processes",
                "attributes": { "pid": "4242", "metadata": { "host": "build-01" } },
                "relationships": {
                    "machine": { "data": { "type": "machines", "id": "mach-1" } }
                }
            }
        })))
        .with_status(201)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(single(process("proc-new")))
        .expect(1)
        .create();
    let ping = service_mock(
        "POST",
        "/v1/processes/proc-ping/actions/ping",
        Matcher::Missing,
        200,
        Some(single(process("proc-ping"))),
    );
    let update = mock("PATCH", "/v1/processes/proc-upd")
        .match_body(Matcher::Json(json!({
            "data": { "type": "processes", "attributes": { "metadata": { "tier": "gold" } } }
        })))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(single(process("proc-upd")))
        .expect(1)
        .create();
    let kill = service_mock(
        "DELETE",
        "/v1/processes/proc-kill",
        Matcher::Missing,
        204,
        None,
    );
    let client = client();
    let processes = client.processes();

    assert_eq!(processes.get("proc-get").await.unwrap().id, "proc-get");
    let spawned = processes
        .spawn(&SpawnProcessRequest {
            pid: "4242".into(),
            machine_id: "mach-1".into(),
            metadata: Some(HashMap::from([("host".to_string(), json!("build-01"))])),
        })
        .await
        .unwrap();
    assert_eq!(spawned.id, "proc-new");
    assert_eq!(processes.ping("proc-ping").await.unwrap().id, "proc-ping");
    let updated = processes
        .update(
            "proc-upd",
            UpdateProcessRequest {
                metadata: Some(HashMap::from([("tier".to_string(), json!("gold"))])),
            },
        )
        .await
        .unwrap();
    assert_eq!(updated.id, "proc-upd");
    processes.kill("proc-kill").await.unwrap();

    get.assert();
    spawn.assert();
    ping.assert();
    update.assert();
    kill.assert();
}

fn second_factor(id: &str, enabled: bool) -> Value {
    let attributes = if enabled {
        json!({ "enabled": true })
    } else {
        json!({
            "enabled": false,
            "uri": "otpauth://totp/Keygen:user?secret=SECRET&issuer=Keygen",
            "secret": "SECRET"
        })
    };
    resource(
        "second-factors",
        id,
        attributes,
        json!({
            "account": linkage("accounts", "account"),
            "user": linkage("users", "user-1")
        }),
    )
}

#[tokio::test]
async fn second_factor_service_lists_gets_and_paginates() {
    let list_mock = service_mock(
        "GET",
        "/v1/users/user-1/second-factors",
        query(&[("page[size]", "5"), ("page[cursor]", "sf-0")]),
        200,
        Some(list(vec![second_factor("sf-1", true)], None)),
    );
    let get = service_mock(
        "GET",
        "/v1/users/user-1/second-factors/sf-2",
        Matcher::Missing,
        200,
        Some(single(second_factor("sf-2", false))),
    );
    let client = client();

    let page = client
        .second_factors()
        .list(
            "user-1",
            Some(&PaginationOptions {
                page_size: Some(5),
                page_cursor: Some("sf-0".into()),
                ..Default::default()
            }),
        )
        .await
        .unwrap();
    let enabled = &page.items()[0];
    assert!(enabled.enabled);
    assert_eq!(enabled.secret, None);
    assert_eq!(enabled.user_id.as_deref(), Some("user-1"));

    let pending = client.second_factors().get("user-1", "sf-2").await.unwrap();
    assert!(!pending.enabled);
    assert_eq!(pending.secret.as_deref(), Some("SECRET"));
    assert!(pending.uri.unwrap().starts_with("otpauth://"));
    list_mock.assert();
    get.assert();
}

#[tokio::test]
async fn second_factor_service_creates_updates_and_deletes_with_user_secrets() {
    let create = mock("POST", "/v1/users/user-1/second-factors")
        .match_body(Matcher::Json(json!({
            "data": { "type": "second-factors" },
            "meta": { "password": "hunter22" }
        })))
        .with_status(201)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(single(second_factor("sf-new", false)))
        .expect(1)
        .create();
    let update = mock("PATCH", "/v1/users/user-1/second-factors/sf-new")
        .match_body(Matcher::Json(json!({
            "data": { "type": "second-factors", "attributes": { "enabled": true } },
            "meta": { "otp": "123456" }
        })))
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(single(second_factor("sf-new", true)))
        .expect(1)
        .create();
    let delete = mock("DELETE", "/v1/users/user-1/second-factors/sf-new")
        .match_body(Matcher::Json(json!({ "meta": { "otp": "654321" } })))
        .with_status(204)
        .expect(1)
        .create();
    let client = client();
    let second_factors = client.second_factors();

    let created = second_factors
        .create(
            "user-1",
            &SecondFactorCredential::Password("hunter22".into()),
        )
        .await
        .unwrap();
    assert!(!created.enabled);
    let enabled = second_factors
        .update("user-1", "sf-new", true, "123456")
        .await
        .unwrap();
    assert!(enabled.enabled);
    second_factors
        .delete("user-1", "sf-new", Some("654321"))
        .await
        .unwrap();

    create.assert();
    update.assert();
    delete.assert();
}

fn event_log(id: &str) -> Value {
    resource(
        "event-logs",
        id,
        json!({
            "event": "license.updated",
            "metadata": { "diff": { "expiry": ["2026-02-01T00:00:00Z", null] } }
        }),
        json!({
            "account": linkage("accounts", "account"),
            "environment": { "data": null },
            "request": linkage("request-logs", "req-1"),
            "whodunnit": linkage("users", "user-1"),
            "resource": linkage("licenses", "lic-1")
        }),
    )
}

#[tokio::test]
async fn event_log_list_sends_filters_and_reads_relationships() {
    let list_mock = service_mock(
        "GET",
        "/v1/event-logs",
        query(&[
            ("date[start]", "2026-01-01T00:00:00Z"),
            ("date[end]", "2026-01-07T00:00:00Z"),
            ("resource[type]", "license"),
            ("resource[id]", "lic-1"),
            ("event", "license.updated"),
            ("page[size]", "10"),
            ("page[cursor]", "evt-0"),
        ]),
        200,
        Some(list(
            vec![event_log("evt-1")],
            Some("/v1/event-logs?page%5Bsize%5D=10&page%5Bcursor%5D=evt-1"),
        )),
    );
    let get = service_mock(
        "GET",
        "/v1/event-logs/evt-2",
        Matcher::Missing,
        200,
        Some(single(event_log("evt-2"))),
    );
    let client = client();

    let page = client
        .event_logs()
        .list(Some(&ListEventLogsOptions {
            page_size: Some(10),
            page_cursor: Some("evt-0".into()),
            date_start: Some("2026-01-01T00:00:00Z".parse().unwrap()),
            date_end: Some("2026-01-07T00:00:00Z".parse().unwrap()),
            resource_type: Some("license".into()),
            resource_id: Some("lic-1".into()),
            event: Some("license.updated".into()),
            ..Default::default()
        }))
        .await
        .unwrap();
    let log = &page.items()[0];
    assert_eq!(log.event, "license.updated");
    assert_eq!(log.request_id.as_deref(), Some("req-1"));
    assert_eq!(
        log.whodunnit,
        Some(ResourceIdentifier {
            resource_type: "users".into(),
            id: "user-1".into()
        })
    );
    assert_eq!(log.resource.as_ref().unwrap().id, "lic-1");
    assert!(log.metadata.as_ref().unwrap().contains_key("diff"));
    assert_eq!(page.next_cursor().as_deref(), Some("evt-1"));

    let log = client.event_logs().get("evt-2").await.unwrap();
    assert_eq!(log.id, "evt-2");
    list_mock.assert();
    get.assert();
}

fn request_log(id: &str, bodies: bool) -> Value {
    let mut attributes = json!({
        "url": "/v1/accounts/account/machines",
        "method": "POST",
        "status": "422",
        "userAgent": "keygen-rs",
        "ip": "192.168.1.1",
        "requestHeaders": { "Accept": "application/json" },
        "responseHeaders": { "Keygen-Edition": "EE" },
        "requestBody": null,
        "responseBody": null,
        "responseSignature": null
    });
    if bodies {
        attributes["requestBody"] = json!("{\"data\":{}}");
        attributes["responseBody"] = json!("{\"errors\":[]}");
    }
    resource(
        "request-logs",
        id,
        attributes,
        json!({
            "account": linkage("accounts", "account"),
            "requestor": linkage("licenses", "lic-1")
        }),
    )
}

#[tokio::test]
async fn request_log_list_sends_filters_and_get_returns_bodies() {
    let list_mock = service_mock(
        "GET",
        "/v1/request-logs",
        query(&[
            ("requestor[type]", "license"),
            ("requestor[id]", "lic-1"),
            ("ip", "192.168.1.1"),
            ("method", "POST"),
            ("status", "422"),
            ("url", "/v1/accounts/account/machines"),
            ("page[size]", "2"),
            ("page[cursor]", ""),
        ]),
        200,
        Some(list(
            vec![request_log("req-1", false)],
            Some("/v1/request-logs?page%5Bsize%5D=2&page%5Bcursor%5D=req-1"),
        )),
    );
    let get = service_mock(
        "GET",
        "/v1/request-logs/req-1",
        Matcher::Missing,
        200,
        Some(single(request_log("req-1", true))),
    );
    let client = client();

    let page = client
        .request_logs()
        .list(Some(&ListRequestLogsOptions {
            page_size: Some(2),
            requestor_type: Some("license".into()),
            requestor_id: Some("lic-1".into()),
            ip: Some("192.168.1.1".into()),
            method: Some("POST".into()),
            status: Some("422".into()),
            url: Some("/v1/accounts/account/machines".into()),
            ..Default::default()
        }))
        .await
        .unwrap();
    let listed = &page.items()[0];
    assert_eq!(listed.status.as_deref(), Some("422"));
    assert_eq!(listed.request_body, None);
    assert_eq!(listed.response_body, None);
    assert_eq!(listed.requestor.as_ref().unwrap().resource_type, "licenses");
    assert_eq!(page.next_cursor().as_deref(), Some("req-1"));

    let full = client.request_logs().get("req-1").await.unwrap();
    assert_eq!(full.request_body.as_deref(), Some("{\"data\":{}}"));
    assert_eq!(full.response_body.as_deref(), Some("{\"errors\":[]}"));
    list_mock.assert();
    get.assert();
}

#[cfg(feature = "unstable-search")]
mod search {
    use super::*;
    use keygen_rs::search::{SearchOp, SearchQuery};

    #[tokio::test]
    async fn search_sends_the_query_body_and_paginates_in_the_query_string() {
        let user = resource(
            "users",
            "user-1",
            json!({ "email": "alice@example.com", "status": "ACTIVE", "role": "user" }),
            json!({}),
        );
        let search = mock("POST", "/v1/search")
            .match_query(query(&[("page[size]", "25"), ("page[cursor]", "")]))
            .match_header("authorization", format!("Bearer {TOKEN}").as_str())
            .match_body(Matcher::Json(json!({
                "meta": {
                    "type": "users",
                    "query": { "email": "alice", "metadata": { "plan": "pro" } },
                    "op": "OR"
                }
            })))
            .with_status(200)
            .with_header("content-type", "application/vnd.api+json")
            .with_body(list(
                vec![user],
                Some("/v1/search?page%5Bsize%5D=25&page%5Bcursor%5D=user-1"),
            ))
            .expect(1)
            .create();

        let page = client()
            .search()
            .users(
                &SearchQuery::new()
                    .term("email", "alice")
                    .metadata("plan", "pro")
                    .op(SearchOp::Or),
                Some(&PaginationOptions {
                    page_size: Some(25),
                    ..Default::default()
                }),
            )
            .await
            .unwrap();

        assert_eq!(page.items()[0].email, "alice@example.com");
        assert_eq!(page.next_cursor().as_deref(), Some("user-1"));
        search.assert();
    }

    #[tokio::test]
    async fn search_defaults_to_and_and_returns_licenses() {
        let license = resource(
            "licenses",
            "lic-1",
            json!({
                "key": "KEY-123",
                "status": "ACTIVE",
                "uses": 0,
                "protected": false,
                "suspended": false,
                "scheme": null,
                "metadata": {}
            }),
            json!({}),
        );
        let search = mock("POST", "/v1/search")
            .match_body(Matcher::Json(json!({
                "meta": { "type": "licenses", "query": { "key": "KEY-" }, "op": "AND" }
            })))
            .with_status(200)
            .with_header("content-type", "application/vnd.api+json")
            .with_body(list(vec![license], None))
            .expect(1)
            .create();

        let page = client()
            .search()
            .licenses(&SearchQuery::new().term("key", "KEY-"), None)
            .await
            .unwrap();

        assert_eq!(page.items()[0].id, "lic-1");
        assert!(!page.has_more());
        search.assert();
    }
}

fn ping_mock(environment: &str, headers: &[(&str, &str)]) -> Mock {
    let mut mock = mock("GET", "/v1/ping")
        .match_header("Keygen-Environment", environment)
        .with_status(200)
        .with_header("content-type", "text/plain");
    for (name, value) in headers {
        mock = mock.with_header(name, value);
    }
    mock.with_body("").expect(1).create()
}

#[tokio::test]
async fn capabilities_of_an_enterprise_server() {
    let ping = ping_mock(
        "caps-ee",
        &[
            ("Keygen-Edition", "EE"),
            ("Keygen-Mode", "multiplayer"),
            ("Keygen-Version", "1.8"),
            ("Keygen-Revision", "abc123"),
        ],
    );

    let capabilities = client_with_environment("caps-ee")
        .capabilities()
        .await
        .unwrap();

    assert_eq!(capabilities.edition, Some(KeygenEdition::Enterprise));
    assert_eq!(capabilities.mode, Some(KeygenMode::Multiplayer));
    assert_eq!(capabilities.api_version.as_deref(), Some("1.8"));
    assert_eq!(capabilities.revision.as_deref(), Some("abc123"));
    assert!(capabilities.is_enterprise());
    assert!(capabilities.is_multiplayer());
    assert!(capabilities.supports_event_logs());
    assert!(capabilities.supports_request_logs());
    assert!(capabilities.supports_environments());
    assert!(capabilities.supports_permissions());
    ping.assert();
}

#[tokio::test]
async fn capabilities_of_a_community_server() {
    let ping = ping_mock(
        "caps-ce",
        &[
            ("Keygen-Edition", "CE"),
            ("Keygen-Mode", "singleplayer"),
            ("Keygen-Version", "1.7"),
        ],
    );

    let capabilities = client_with_environment("caps-ce")
        .capabilities()
        .await
        .unwrap();

    assert_eq!(capabilities.edition, Some(KeygenEdition::Community));
    assert_eq!(capabilities.mode, Some(KeygenMode::Singleplayer));
    assert!(capabilities.is_community());
    assert!(!capabilities.is_multiplayer());
    assert!(!capabilities.supports_event_logs());
    assert!(!capabilities.supports_request_logs());
    assert!(!capabilities.supports_environments());
    assert!(!capabilities.supports_permissions());
    ping.assert();
}

#[tokio::test]
async fn capabilities_without_headers_are_unknown() {
    let ping = ping_mock("caps-none", &[("Keygen-Edition", "XE")]);

    let capabilities = client_with_environment("caps-none")
        .capabilities()
        .await
        .unwrap();

    assert_eq!(capabilities.edition, None);
    assert_eq!(capabilities.mode, None);
    assert_eq!(capabilities.api_version, None);
    assert!(!capabilities.is_enterprise());
    assert!(!capabilities.is_community());
    assert!(!capabilities.supports_event_logs());
    ping.assert();
}

#[tokio::test]
async fn rate_limited_responses_surface_retry_after_and_limits() {
    let limited = mock("GET", "/v1/event-logs/evt-limited")
        .with_status(429)
        .with_header("content-type", "application/vnd.api+json")
        .with_header("X-RateLimit-Window", "30s")
        .with_header("X-RateLimit-Count", "61")
        .with_header("X-RateLimit-Limit", "60")
        .with_header("X-RateLimit-Remaining", "0")
        .with_header("X-RateLimit-Reset", "1767225630")
        .with_header("Retry-After", "17")
        .with_body(r#"{"errors":[{"code":"TOO_MANY_REQUESTS"}]}"#)
        .expect(1)
        .create();
    let client = client();

    let error = client.event_logs().get("evt-limited").await.unwrap_err();

    assert!(matches!(error, Error::RateLimitExceeded { .. }));
    assert_eq!(error.retry_after(), Some(Duration::from_secs(17)));
    let rate_limit = error.rate_limit().unwrap();
    assert_eq!(rate_limit.window.as_deref(), Some("30s"));
    assert_eq!(rate_limit.count, Some(61));
    assert_eq!(rate_limit.limit, Some(60));
    assert_eq!(rate_limit.remaining, Some(0));
    assert_eq!(rate_limit.reset, Some(1_767_225_630));
    assert!(rate_limit.is_exhausted());
    let last = client.last_rate_limit().unwrap();
    assert_eq!(last.retry_after, Some(17));
    limited.assert();
}

#[tokio::test]
async fn successful_responses_record_the_rate_limit_status() {
    let ok = mock("GET", "/v1/event-logs/evt-ok")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_header("X-RateLimit-Window", "5m")
        .with_header("X-RateLimit-Count", "3")
        .with_header("X-RateLimit-Limit", "500")
        .with_header("X-RateLimit-Remaining", "497")
        .with_header("X-RateLimit-Reset", "1767225900")
        .with_body(single(event_log("evt-ok")))
        .expect(1)
        .create();
    let client = client();
    assert_eq!(client.last_rate_limit(), None);

    client.event_logs().get("evt-ok").await.unwrap();

    let status = client.last_rate_limit().unwrap();
    assert_eq!(status.window.as_deref(), Some("5m"));
    assert_eq!(status.remaining, Some(497));
    assert_eq!(status.retry_after, None);
    assert!(!status.is_exhausted());
    assert_eq!(status.reset_at().unwrap().timestamp(), 1_767_225_900);
    ok.assert();
}

#[tokio::test]
async fn non_rate_limit_errors_have_no_rate_limit() {
    let missing = mock("GET", "/v1/event-logs/evt-missing")
        .with_status(404)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(r#"{"errors":[{"code":"NOT_FOUND"}]}"#)
        .expect(1)
        .create();

    let error = client().event_logs().get("evt-missing").await.unwrap_err();

    assert_eq!(error.rate_limit(), None);
    assert_eq!(error.retry_after(), None);
    missing.assert();
}
