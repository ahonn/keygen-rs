//! Instance-client services: every request uses the client's own account,
//! token and environment, and list calls return a cursor [`Page`].
#![cfg(feature = "token")]

use keygen_rs::machine::Machine;
use keygen_rs::product::ListProductsOptions;
use keygen_rs::{KeygenClient, Page};
use mockito::{mock, Matcher, Mock};
use serde_json::{json, Value};

const TOKEN: &str = "service-token";
const ENVIRONMENT: &str = "service-env";
const TIMESTAMP: &str = "2026-01-01T00:00:00Z";

fn client() -> KeygenClient {
    client_for("account", TOKEN, ENVIRONMENT)
}

fn client_for(account: &str, token: &str, environment: &str) -> KeygenClient {
    KeygenClient::builder()
        .account(account)
        .api_url(mockito::server_url())
        .token(token)
        .environment(environment)
        .build()
        .unwrap()
}

fn resource(kind: &str, id: &str, mut attributes: Value) -> Value {
    attributes["created"] = json!(TIMESTAMP);
    attributes["updated"] = json!(TIMESTAMP);
    json!({ "type": kind, "id": id, "attributes": attributes, "relationships": {} })
}

fn single(data: Value) -> String {
    json!({ "data": data }).to_string()
}

fn list(data: Vec<Value>, next: Option<&str>) -> String {
    json!({ "data": data, "meta": {}, "links": { "self": "/v1/x", "next": next } }).to_string()
}

/// A mock that only answers requests carrying this test's token and environment.
fn service_mock(method: &str, path: &str, status: usize, body: Option<String>) -> Mock {
    let mock = mock(method, path)
        .match_query(Matcher::Any)
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

fn product(id: &str) -> Value {
    resource("products", id, json!({ "name": "Product" }))
}

fn policy(id: &str) -> Value {
    resource(
        "policies",
        id,
        json!({
            "name": "Policy",
            "strict": false,
            "floating": false,
            "requireHeartbeat": false,
            "expirationStrategy": "RESTRICT_ACCESS",
            "authenticationStrategy": "TOKEN",
            "machineLeasingStrategy": "PER_LICENSE",
            "processLeasingStrategy": "PER_MACHINE",
            "overageStrategy": "NO_OVERAGE",
            "transferStrategy": "KEEP_EXPIRY",
            "encrypted": false,
            "protected": false,
            "requireCheckIn": false,
            "usePool": false
        }),
    )
}

fn machine(id: &str) -> Value {
    resource(
        "machines",
        id,
        json!({
            "fingerprint": "fingerprint",
            "requireHeartbeat": false,
            "heartbeatStatus": "NOT_STARTED"
        }),
    )
}

fn user(id: &str) -> Value {
    resource(
        "users",
        id,
        json!({ "email": "user@example.com", "status": "ACTIVE", "role": "user" }),
    )
}

fn release(id: &str) -> Value {
    resource(
        "releases",
        id,
        json!({ "version": "1.0.0", "channel": "stable", "status": "PUBLISHED" }),
    )
}

fn artifact(id: &str) -> Value {
    resource(
        "artifacts",
        id,
        json!({ "filename": "app.zip", "status": "UPLOADED" }),
    )
}

fn webhook_event(id: &str) -> Value {
    resource(
        "webhook-events",
        id,
        json!({
            "endpoint": "https://example.com/hook",
            "payload": "{}",
            "event": "license.created",
            "status": "DELIVERED"
        }),
    )
}

fn ids<T>(page: &Page<T>, id: impl Fn(&T) -> &str) -> Vec<String> {
    page.items()
        .iter()
        .map(|item| id(item).to_string())
        .collect()
}

#[tokio::test]
async fn two_clients_keep_accounts_and_environments_isolated() {
    let acme = mock("GET", "/v1/policies")
        .match_query(Matcher::Any)
        .match_header("authorization", "Bearer acme-token")
        .match_header("Keygen-Environment", "acme-production")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(list(vec![policy("acme-policy")], None))
        .expect(1)
        .create();
    let globex = mock("GET", "/v1/policies")
        .match_query(Matcher::Any)
        .match_header("authorization", "Bearer globex-token")
        .match_header("Keygen-Environment", "globex-staging")
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(list(vec![policy("globex-policy")], None))
        .expect(1)
        .create();
    let acme_delete = mock("DELETE", "/v1/policies/acme-policy")
        .match_header("authorization", "Bearer acme-token")
        .match_header("Keygen-Environment", "acme-production")
        .with_status(204)
        .expect(1)
        .create();

    let acme_client = client_for("acme", "acme-token", "acme-production");
    let globex_client = client_for("globex", "globex-token", "globex-staging");

    let acme_policies = acme_client.policies();
    let globex_policies = globex_client.policies();
    let (acme_page, globex_page) =
        tokio::join!(acme_policies.list(None), globex_policies.list(None));
    acme_client.policies().delete("acme-policy").await.unwrap();

    assert_eq!(
        acme_client.config().environment.as_deref(),
        Some("acme-production")
    );
    assert_eq!(
        globex_client.config().environment.as_deref(),
        Some("globex-staging")
    );
    assert_eq!(ids(&acme_page.unwrap(), |p| &p.id), ["acme-policy"]);
    assert_eq!(ids(&globex_page.unwrap(), |p| &p.id), ["globex-policy"]);
    acme.assert();
    globex.assert();
    acme_delete.assert();
}

#[tokio::test]
async fn product_list_follows_cursor_pages() {
    let first = mock("GET", "/v1/products")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("page[size]".into(), "1".into()),
            Matcher::UrlEncoded("page[cursor]".into(), "".into()),
        ]))
        .match_header("Keygen-Environment", ENVIRONMENT)
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(list(
            vec![product("prod-1")],
            Some("/v1/products?page%5Bsize%5D=1&page%5Bcursor%5D=prod-1"),
        ))
        .expect(1)
        .create();
    let last = mock("GET", "/v1/products")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("page[size]".into(), "1".into()),
            Matcher::UrlEncoded("page[cursor]".into(), "prod-1".into()),
        ]))
        .match_header("Keygen-Environment", ENVIRONMENT)
        .with_status(200)
        .with_header("content-type", "application/vnd.api+json")
        .with_body(list(vec![product("prod-2")], None))
        .expect(1)
        .create();
    let client = client();

    let page = client
        .products()
        .list(Some(&ListProductsOptions {
            page_size: Some(1),
            ..Default::default()
        }))
        .await
        .unwrap();
    assert_eq!(ids(&page, |p| &p.id), ["prod-1"]);
    assert!(page.has_more());
    let cursor = page.next_cursor().expect("first page has a next cursor");
    assert_eq!(cursor, "prod-1");

    let page = client
        .products()
        .list(Some(&ListProductsOptions {
            page_size: Some(1),
            page_cursor: Some(cursor),
            ..Default::default()
        }))
        .await
        .unwrap();
    assert_eq!(ids(&page, |p| &p.id), ["prod-2"]);
    assert!(!page.has_more());
    assert_eq!(page.next_cursor(), None);
    first.assert();
    last.assert();
}

#[tokio::test]
async fn list_rejects_page_sizes_outside_the_api_range() {
    let error = client()
        .products()
        .list(Some(&ListProductsOptions {
            page_size: Some(101),
            ..Default::default()
        }))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        keygen_rs::errors::Error::InvalidPageSize(101)
    ));
}

#[tokio::test]
async fn product_service_deletes() {
    let m = service_mock("DELETE", "/v1/products/prod-del", 204, None);
    client().products().delete("prod-del").await.unwrap();
    m.assert();
}

#[tokio::test]
async fn machine_service_lists_and_resets() {
    let list_mock = service_mock(
        "GET",
        "/v1/machines",
        200,
        Some(list(vec![machine("machine-list")], None)),
    );
    let reset = service_mock(
        "POST",
        "/v1/machines/machine-reset/actions/reset",
        200,
        Some(single(machine("machine-reset"))),
    );
    let client = client();

    let page = client.machines().list(None).await.unwrap();
    let machine = client.machines().reset("machine-reset").await.unwrap();

    assert_eq!(ids(&page, |m| &m.id), ["machine-list"]);
    assert_eq!(machine.id, "machine-reset");
    list_mock.assert();
    reset.assert();
}

#[tokio::test]
async fn machines_from_a_service_keep_the_client_for_follow_up_calls() {
    let get = service_mock(
        "GET",
        "/v1/machines/machine-bound",
        200,
        Some(single(machine("machine-bound"))),
    );
    let deactivate = service_mock("DELETE", "/v1/machines/machine-bound", 204, None);

    let machine: Machine = client().machines().get("machine-bound").await.unwrap();
    machine.deactivate().await.unwrap();

    get.assert();
    deactivate.assert();
}

#[tokio::test]
async fn user_service_lists_and_bans() {
    let list_mock = service_mock("GET", "/v1/users", 200, Some(list(vec![user("u-1")], None)));
    let ban = service_mock(
        "POST",
        "/v1/users/u-ban/actions/ban",
        200,
        Some(single(user("u-ban"))),
    );
    let client = client();

    let page = client.users().list(None).await.unwrap();
    let banned = client.users().ban("u-ban").await.unwrap();

    assert_eq!(ids(&page, |u| &u.id), ["u-1"]);
    assert_eq!(banned.id, "u-ban");
    list_mock.assert();
    ban.assert();
}

#[tokio::test]
async fn group_service_lists_and_deletes() {
    let group = resource("groups", "grp-1", json!({ "name": "Group" }));
    let list_mock = service_mock("GET", "/v1/groups", 200, Some(list(vec![group], None)));
    let delete = service_mock("DELETE", "/v1/groups/grp-del", 204, None);
    let client = client();

    let page = client.groups().list(None).await.unwrap();
    client.groups().delete("grp-del").await.unwrap();

    assert_eq!(ids(&page, |g| &g.id), ["grp-1"]);
    list_mock.assert();
    delete.assert();
}

#[tokio::test]
async fn entitlement_service_lists_and_deletes() {
    let entitlement = resource("entitlements", "ent-1", json!({ "code": "FEATURE" }));
    let list_mock = service_mock(
        "GET",
        "/v1/entitlements",
        200,
        Some(list(vec![entitlement], None)),
    );
    let delete = service_mock("DELETE", "/v1/entitlements/ent-del", 204, None);
    let client = client();

    let page = client.entitlements().list(None).await.unwrap();
    client.entitlements().delete("ent-del").await.unwrap();

    assert_eq!(ids(&page, |e| &e.id), ["ent-1"]);
    list_mock.assert();
    delete.assert();
}

#[tokio::test]
async fn token_service_lists_and_revokes() {
    let token = resource("tokens", "tok-1", json!({ "kind": "product-token" }));
    let list_mock = service_mock("GET", "/v1/tokens", 200, Some(list(vec![token], None)));
    let revoke = service_mock("DELETE", "/v1/tokens/tok-revoke", 204, None);
    let client = client();

    let page = client.tokens().list(None).await.unwrap();
    client.tokens().revoke("tok-revoke").await.unwrap();

    assert_eq!(ids(&page, |t| &t.id), ["tok-1"]);
    list_mock.assert();
    revoke.assert();
}

#[tokio::test]
async fn environment_service_lists_and_deletes() {
    let environment = resource(
        "environments",
        "env-1",
        json!({ "name": "Staging", "code": "staging", "isolationStrategy": "ISOLATED" }),
    );
    let list_mock = service_mock(
        "GET",
        "/v1/environments",
        200,
        Some(list(vec![environment], None)),
    );
    let delete = service_mock("DELETE", "/v1/environments/env-del", 204, None);
    let client = client();

    let page = client.environments().list(None).await.unwrap();
    client.environments().delete("env-del").await.unwrap();

    assert_eq!(ids(&page, |e| &e.id), ["env-1"]);
    list_mock.assert();
    delete.assert();
}

#[tokio::test]
async fn component_service_lists_and_deletes() {
    let component = resource(
        "components",
        "cmp-1",
        json!({ "fingerprint": "gpu", "name": "GPU" }),
    );
    let list_mock = service_mock(
        "GET",
        "/v1/components",
        200,
        Some(list(vec![component], None)),
    );
    let delete = service_mock("DELETE", "/v1/components/cmp-del", 204, None);
    let client = client();

    let page = client.components().list(None).await.unwrap();
    client.components().delete("cmp-del").await.unwrap();

    assert_eq!(ids(&page, |c| &c.id), ["cmp-1"]);
    list_mock.assert();
    delete.assert();
}

#[tokio::test]
async fn release_service_lists_and_publishes() {
    let list_mock = service_mock(
        "GET",
        "/v1/releases",
        200,
        Some(list(vec![release("rel-1")], None)),
    );
    let publish = service_mock(
        "POST",
        "/v1/releases/rel-pub/actions/publish",
        200,
        Some(single(release("rel-pub"))),
    );
    let client = client();

    let page = client.releases().list(None).await.unwrap();
    let published = client.releases().publish("rel-pub").await.unwrap();

    assert_eq!(ids(&page, |r| &r.id), ["rel-1"]);
    assert_eq!(published.id, "rel-pub");
    list_mock.assert();
    publish.assert();
}

#[tokio::test]
async fn package_service_lists_and_deletes() {
    let package = resource("packages", "pkg-1", json!({ "name": "App", "key": "app" }));
    let list_mock = service_mock("GET", "/v1/packages", 200, Some(list(vec![package], None)));
    let delete = service_mock("DELETE", "/v1/packages/pkg-del", 204, None);
    let client = client();

    let page = client.packages().list(None).await.unwrap();
    client.packages().delete("pkg-del").await.unwrap();

    assert_eq!(ids(&page, |p| &p.id), ["pkg-1"]);
    list_mock.assert();
    delete.assert();
}

#[tokio::test]
async fn artifact_service_lists_and_yanks() {
    let list_mock = service_mock(
        "GET",
        "/v1/artifacts",
        200,
        Some(list(vec![artifact("art-1")], None)),
    );
    let yank = service_mock(
        "POST",
        "/v1/artifacts/art-yank/actions/yank",
        200,
        Some(single(artifact("art-yank"))),
    );
    let client = client();

    let page = client.artifacts().list(None).await.unwrap();
    let yanked = client.artifacts().yank("art-yank").await.unwrap();

    assert_eq!(ids(&page, |a| &a.id), ["art-1"]);
    assert_eq!(yanked.id, "art-yank");
    list_mock.assert();
    yank.assert();
}

#[tokio::test]
async fn platform_arch_and_channel_services_list_and_get() {
    let key = |kind: &str, id: &str| resource(kind, id, json!({ "key": id }));
    let mocks = [
        service_mock(
            "GET",
            "/v1/platforms",
            200,
            Some(list(vec![key("platforms", "macos")], None)),
        ),
        service_mock(
            "GET",
            "/v1/platforms/linux",
            200,
            Some(single(key("platforms", "linux"))),
        ),
        service_mock(
            "GET",
            "/v1/arches",
            200,
            Some(list(vec![key("arches", "arm64")], None)),
        ),
        service_mock(
            "GET",
            "/v1/arches/x86",
            200,
            Some(single(key("arches", "x86"))),
        ),
        service_mock(
            "GET",
            "/v1/channels",
            200,
            Some(list(vec![key("channels", "stable")], None)),
        ),
        service_mock(
            "GET",
            "/v1/channels/beta",
            200,
            Some(single(key("channels", "beta"))),
        ),
    ];
    let client = client();

    let platforms = client.platforms().list(None).await.unwrap();
    let platform = client.platforms().get("linux").await.unwrap();
    let arches = client.arches().list(None).await.unwrap();
    let arch = client.arches().get("x86").await.unwrap();
    let channels = client.channels().list(None).await.unwrap();
    let channel = client.channels().get("beta").await.unwrap();

    assert_eq!(ids(&platforms, |p| &p.id), ["macos"]);
    assert_eq!(platform.id, "linux");
    assert_eq!(ids(&arches, |a| &a.id), ["arm64"]);
    assert_eq!(arch.id, "x86");
    assert_eq!(ids(&channels, |c| &c.id), ["stable"]);
    assert_eq!(channel.id, "beta");
    for m in mocks {
        m.assert();
    }
}

#[tokio::test]
async fn webhook_endpoint_service_lists_and_deletes() {
    let endpoint = resource(
        "webhook-endpoints",
        "hook-1",
        json!({ "url": "https://example.com/hook", "signatureAlgorithm": "ed25519" }),
    );
    let list_mock = service_mock(
        "GET",
        "/v1/webhook-endpoints",
        200,
        Some(list(vec![endpoint], None)),
    );
    let delete = service_mock("DELETE", "/v1/webhook-endpoints/hook-del", 204, None);
    let client = client();

    let page = client.webhook_endpoints().list(None).await.unwrap();
    client.webhook_endpoints().delete("hook-del").await.unwrap();

    assert_eq!(ids(&page, |e| &e.id), ["hook-1"]);
    list_mock.assert();
    delete.assert();
}

#[tokio::test]
async fn webhook_event_service_lists_and_retries() {
    let list_mock = service_mock(
        "GET",
        "/v1/webhook-events",
        200,
        Some(list(vec![webhook_event("evt-1")], None)),
    );
    let retry = service_mock(
        "POST",
        "/v1/webhook-events/evt-retry/actions/retry",
        200,
        Some(single(webhook_event("evt-retry"))),
    );
    let client = client();

    let page = client.webhook_events().list(None).await.unwrap();
    let retried = client.webhook_events().retry("evt-retry").await.unwrap();

    assert_eq!(ids(&page, |e| &e.id), ["evt-1"]);
    assert_eq!(retried.id, "evt-retry");
    list_mock.assert();
    retry.assert();
}

#[tokio::test]
async fn ping_uses_the_client_environment() {
    let ping = mock("GET", "/v1/ping")
        .match_header("Keygen-Environment", "ping-env")
        .with_status(200)
        .with_header("content-type", "text/plain")
        .with_body("ok")
        .expect(1)
        .create();

    let response = client_for("account", TOKEN, "ping-env")
        .ping()
        .await
        .unwrap();

    assert_eq!(response.message, "ok");
    ping.assert();
}
