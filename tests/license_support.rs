use keygen_rs::license::{
    LicenseValidationCode, LicenseValidationMeta, LicenseValidationScope, PageLinks, ResourcePage,
};
use serde_json::json;

#[test]
fn resource_page_exposes_typed_links_and_next_cursor() {
    let page = ResourcePage::<String> {
        data: vec!["license-1".into()],
        meta: Some(json!({ "requestId": "request-1" })),
        links: Some(json!({
            "self": "/v1/licenses?page[size]=1",
            "next": "/v1/licenses?page[size]=1&page[cursor]=license-1",
            "future": "preserved"
        })),
    };

    let links = page
        .typed_links()
        .expect("links should be valid")
        .expect("links should be present");

    assert_eq!(links.self_url(), Some("/v1/licenses?page[size]=1"));
    assert_eq!(links.next_cursor().as_deref(), Some("license-1"));
    assert_eq!(links.extensions()["future"], "preserved");
    assert_eq!(page.next_cursor().as_deref(), Some("license-1"));
}

#[test]
fn page_links_support_null_next_and_absolute_urls() {
    let last_page: PageLinks = serde_json::from_value(json!({
        "self": "/v1/licenses",
        "next": null
    }))
    .expect("links should deserialize");
    let next_page: PageLinks = serde_json::from_value(json!({
        "next": "https://api.keygen.sh/v1/licenses?page%5Bcursor%5D=license-2"
    }))
    .expect("links should deserialize");

    assert!(!last_page.has_next_page());
    assert_eq!(last_page.next_cursor(), None);
    assert_eq!(next_page.next_cursor().as_deref(), Some("license-2"));
}

#[test]
fn validation_meta_classifies_documented_codes_without_losing_raw_values() {
    let meta = LicenseValidationMeta {
        ts: "2026-01-01T00:00:00Z".parse().unwrap(),
        valid: false,
        detail: "fingerprint does not match".into(),
        code: "FINGERPRINT_SCOPE_MISMATCH".into(),
        scope: LicenseValidationScope::default(),
        nonce: None,
    };

    assert_eq!(
        meta.code_kind(),
        LicenseValidationCode::FingerprintScopeMismatch
    );
    assert_eq!(meta.code, "FINGERPRINT_SCOPE_MISMATCH");
    assert_eq!(
        LicenseValidationCode::parse("FUTURE_VALIDATION_CODE"),
        LicenseValidationCode::Unknown
    );
}
