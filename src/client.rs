//! HTTP client for the Keygen API.
//!
//! This module provides the low-level HTTP client used to communicate with the Keygen API.
//! It handles authentication, request signing verification, and error handling.

use crate::api_version::ApiContractVersion;
use crate::config::get_config;
use crate::errors::Error;
use crate::verifier::Verifier;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client as ReqwestClient, Request, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
use url::Url;

#[derive(Clone, Debug)]
pub struct Client {
    inner: ReqwestClient,
    options: ClientOptions,
}

#[derive(Clone, Debug)]
pub struct ClientOptions {
    pub account: String,
    pub environment: Option<String>,
    pub license_key: Option<String>,
    pub token: Option<String>,
    pub user_agent: Option<String>,
    pub api_url: String,
    pub api_version: ApiContractVersion,
    pub api_prefix: String,
    pub verify_keygen_signature: bool,
    pub public_key: Option<String>,
}

#[derive(Debug)]
pub struct Response<T> {
    #[allow(dead_code)]
    pub status: StatusCode,
    #[allow(dead_code)]
    pub headers: HeaderMap,
    pub body: T,
}

/// Empty response body for API calls that return no content
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmptyResponse;

impl From<crate::config::KeygenConfig> for ClientOptions {
    fn from(config: crate::config::KeygenConfig) -> Self {
        Self {
            account: config.account,
            environment: config.environment,
            #[cfg(feature = "license-key")]
            license_key: config.license_key,
            #[cfg(not(feature = "license-key"))]
            license_key: None,
            #[cfg(feature = "token")]
            token: config.token,
            #[cfg(not(feature = "token"))]
            token: None,
            user_agent: config.user_agent,
            api_url: config.api_url,
            api_version: config.api_version,
            api_prefix: config.api_prefix,
            #[cfg(feature = "license-key")]
            verify_keygen_signature: config.verify_keygen_signature.unwrap_or(true),
            #[cfg(not(feature = "license-key"))]
            verify_keygen_signature: true,
            #[cfg(feature = "license-key")]
            public_key: config.public_key,
            #[cfg(not(feature = "license-key"))]
            public_key: None,
        }
    }
}

impl Client {
    /// Creates a client using the global configuration.
    ///
    /// This is a convenience method that reads from the global config set via `set_config`.
    pub fn from_global_config() -> Result<Self, Error> {
        let config = get_config()?;
        Self::new(ClientOptions::from(config))
    }

    /// Creates a new client with the specified options.
    pub fn new(options: ClientOptions) -> Result<Self, Error> {
        let builder = ReqwestClient::builder();
        #[cfg(not(target_arch = "wasm32"))]
        let builder = builder.timeout(Duration::from_secs(30));
        let client = builder
            .build()
            .map_err(|e| Error::UnexpectedError(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            inner: client,
            options,
        })
    }

    pub fn set_query<T: Serialize + ?Sized>(
        &self,
        request: Request,
        query: &T,
    ) -> Result<Request, Error> {
        let mut request = request;
        let query_string = serde_urlencoded::to_string(query)?;
        let url = request.url_mut();
        url.set_query(Some(&query_string));
        Ok(request)
    }

    pub async fn get<T, U>(&self, path: &str, params: Option<&T>) -> Result<Response<U>, Error>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned + Serialize,
    {
        self.send(self.new_request(reqwest::Method::GET, path, params)?)
            .await
    }

    pub async fn post<T, U, Q>(
        &self,
        path: &str,
        body: Option<&T>,
        query: Option<&Q>,
    ) -> Result<Response<U>, Error>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned + Serialize,
        Q: Serialize + ?Sized,
    {
        let mut request = self.new_request(reqwest::Method::POST, path, body)?;
        if let Some(q) = query {
            request = self.set_query(request, q)?;
        }
        self.send(request).await
    }

    #[allow(dead_code)]
    pub async fn put<T, U, Q>(
        &self,
        path: &str,
        body: Option<&T>,
        query: Option<&Q>,
    ) -> Result<Response<U>, Error>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned + Serialize,
        Q: Serialize + ?Sized,
    {
        let mut request = self.new_request(reqwest::Method::PUT, path, body)?;
        if let Some(q) = query {
            request = self.set_query(request, q)?;
        }
        self.send(request).await
    }

    #[allow(dead_code)]
    pub async fn patch<T, U, Q>(
        &self,
        path: &str,
        body: Option<&T>,
        query: Option<&Q>,
    ) -> Result<Response<U>, Error>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned + Serialize,
        Q: Serialize + ?Sized,
    {
        let mut request = self.new_request(reqwest::Method::PATCH, path, body)?;
        if let Some(q) = query {
            request = self.set_query(request, q)?;
        }
        self.send(request).await
    }

    pub async fn delete<T, U>(&self, path: &str, params: Option<&T>) -> Result<Response<U>, Error>
    where
        T: Serialize + ?Sized,
        U: DeserializeOwned + Serialize,
    {
        self.send(self.new_request(reqwest::Method::DELETE, path, params)?)
            .await
    }

    /// Get request that returns plain text response (for ping endpoint)
    pub async fn get_text(&self, path: &str) -> Result<Response<String>, Error> {
        let request = self.new_request_no_version(reqwest::Method::GET, path, None::<&()>)?;
        self.send_text(request).await
    }

    pub async fn get_text_with_params<T: Serialize + ?Sized>(
        &self,
        path: &str,
        params: Option<&T>,
    ) -> Result<Response<String>, Error> {
        let request = self.new_request(reqwest::Method::GET, path, params)?;
        self.send_text(request).await
    }

    fn new_request<T: Serialize + ?Sized>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<&T>,
    ) -> Result<Request, Error> {
        self.build_request(method, path, params, true)
    }

    fn new_request_no_version<T: Serialize + ?Sized>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<&T>,
    ) -> Result<Request, Error> {
        self.build_request(method, path, params, false)
    }

    pub(crate) fn build_url(&self, path: &str) -> Result<Url, Error> {
        let mut url = Url::parse(&self.options.api_url)?;

        if self.options.api_url == "https://api.keygen.sh" {
            url.path_segments_mut()
                .map_err(|_| Error::InvalidUrl)?
                .push(self.options.api_prefix.as_str())
                .push("accounts")
                .push(self.options.account.as_str())
                .extend(path.split('/'));
        } else {
            url.path_segments_mut()
                .map_err(|_| Error::InvalidUrl)?
                .push(self.options.api_prefix.as_str())
                .extend(path.split('/'));
        }

        Ok(url)
    }

    pub(crate) fn build_request<T: Serialize + ?Sized>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<&T>,
        include_version: bool,
    ) -> Result<Request, Error> {
        let mut url = self.build_url(path)?;

        if method == reqwest::Method::GET {
            if let Some(params) = params {
                let query = serde_urlencoded::to_string(params)?;
                url.set_query(Some(&query));
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/vnd.api+json"));
        if params.is_some() {
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/vnd.api+json"),
            );
        }
        if let Some(user_agent) = &self.options.user_agent {
            headers.insert(USER_AGENT, HeaderValue::from_str(user_agent)?);
        }

        if let Some(env) = &self.options.environment {
            headers.insert("Keygen-Environment", HeaderValue::from_str(env)?);
        }

        if include_version {
            headers.insert(
                "Keygen-Version",
                HeaderValue::from_static(self.options.api_version.as_str()),
            );
        }

        if let Some(key) = &self.options.license_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("License {key}"))?,
            );
        } else if let Some(token) = &self.options.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))?,
            );
        }

        let mut request = self.inner.request(method.clone(), url).headers(headers);

        if method != reqwest::Method::GET && params.is_some() {
            request = request.json(&json!(params));
        }
        Ok(request.build()?)
    }

    async fn send<U: DeserializeOwned + Serialize>(
        &self,
        request: Request,
    ) -> Result<Response<U>, Error> {
        let method = request.method().as_str().to_owned();
        let url = request.url().clone();
        let host = match (url.host_str(), url.port()) {
            (Some(h), Some(p)) => format!("{h}:{p}"),
            (Some(h), None) => h.to_string(),
            _ => "api.keygen.sh".to_string(),
        };
        let response = self.inner.execute(request).await?;

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.bytes().await?;

        self.verify_response_signature(&headers, &bytes, &method, &url, &host)?;

        if status.is_client_error() || status.is_server_error() {
            let error_body = serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).into_owned().into());
            return Err(self.handle_error(status, &headers, error_body));
        }

        let body: U = if status == StatusCode::NO_CONTENT {
            serde_json::from_value(serde_json::Value::Null)?
        } else {
            serde_json::from_slice(&bytes)?
        };

        Ok(Response {
            status,
            headers,
            body,
        })
    }

    async fn send_text(&self, request: Request) -> Result<Response<String>, Error> {
        let method = request.method().as_str().to_owned();
        let url = request.url().clone();
        let host = match (url.host_str(), url.port()) {
            (Some(h), Some(p)) => format!("{h}:{p}"),
            (Some(h), None) => h.to_string(),
            _ => "api.keygen.sh".to_string(),
        };

        let response = self.inner.execute(request).await?;

        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.bytes().await?;

        self.verify_response_signature(&headers, &bytes, &method, &url, &host)?;

        if status.is_client_error() || status.is_server_error() {
            let error_body = serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).into_owned().into());
            return Err(self.handle_error(status, &headers, error_body));
        }

        let text = String::from_utf8_lossy(&bytes).into_owned();

        Ok(Response {
            status,
            headers,
            body: text,
        })
    }

    fn verify_response_signature(
        &self,
        headers: &HeaderMap,
        body: &[u8],
        method: &str,
        url: &Url,
        host: &str,
    ) -> Result<(), Error> {
        if !self.options.verify_keygen_signature {
            return Ok(());
        }

        let Some(public_key) = &self.options.public_key else {
            return Ok(());
        };

        let full_path = match url.query() {
            Some(query) => format!("{}?{query}", url.path()),
            None => url.path().to_string(),
        };
        Verifier::new(public_key.clone())
            .verify_keygen_signature(headers, body, method, &full_path, host)
    }

    fn handle_error(
        &self,
        status: StatusCode,
        headers: &HeaderMap,
        body: serde_json::Value,
    ) -> Error {
        match status {
            StatusCode::TOO_MANY_REQUESTS => self.handle_rate_limit_error(headers),
            _ => Error::Api(crate::errors::ApiErrorDocument::from_response(status, body)),
        }
    }

    fn handle_rate_limit_error(&self, headers: &HeaderMap) -> Error {
        let window = headers
            .get("X-RateLimit-Window")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let retry_after = headers
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let count = headers
            .get("X-RateLimit-Count")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let limit = headers
            .get("X-RateLimit-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let remaining = headers
            .get("X-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let reset = headers
            .get("X-RateLimit-Reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        Error::RateLimitExceeded {
            window: window.to_string(),
            count,
            limit,
            remaining,
            reset,
            retry_after,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_version::ApiContractVersion;
    use mockito::{mock, server_url};
    use serde_json::json;

    fn create_test_client() -> Client {
        Client::new(ClientOptions {
            account: "test_account".to_string(),
            environment: None,
            license_key: Some("test_license_key".to_string()),
            token: None,
            user_agent: Some("test_user_agent".to_string()),
            api_url: server_url(),
            api_version: ApiContractVersion::V1_8,
            api_prefix: "v1".to_string(),
            public_key: None,
            verify_keygen_signature: true,
        })
        .expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_get_request() {
        let _m = mock("GET", "/v1/test_path")
            .match_header("Keygen-Version", "1.8")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": {"id": "123", "type": "test"}}"#)
            .create();

        let client = create_test_client();
        let response: Response<serde_json::Value> =
            client.get("test_path", None::<&()>).await.unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["data"]["id"], "123");
    }

    #[tokio::test]
    async fn test_post_request() {
        let _m = mock("POST", "/v1/test_path")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": {"id": "456", "type": "test"}}"#)
            .create();

        let client = create_test_client();
        let params = json!({"name": "Test"});
        let response: Response<serde_json::Value> = client
            .post("test_path", Some(&params), None::<&()>)
            .await
            .unwrap();

        assert_eq!(response.status, StatusCode::CREATED);
        assert_eq!(response.body["data"]["id"], "456");
    }

    #[tokio::test]
    async fn test_put_request() {
        let _m = mock("PUT", "/v1/test_path/123")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"data": {"id": "123", "type": "test", "attributes": {"name": "Updated"}}}"#,
            )
            .create();

        let client = create_test_client();
        let params = json!({"name": "Updated"});
        let response: Response<serde_json::Value> = client
            .put("test_path/123", Some(&params), None::<&()>)
            .await
            .unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["data"]["id"], "123");
        assert_eq!(response.body["data"]["attributes"]["name"], "Updated");
    }

    #[tokio::test]
    async fn test_patch_request() {
        let _m = mock("PATCH", "/v1/test_path/456")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"data": {"id": "456", "type": "test", "attributes": {"status": "active"}}}"#,
            )
            .create();

        let client = create_test_client();
        let params = json!({"status": "active"});
        let response: Response<serde_json::Value> = client
            .patch("test_path/456", Some(&params), None::<&()>)
            .await
            .unwrap();

        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body["data"]["id"], "456");
        assert_eq!(response.body["data"]["attributes"]["status"], "active");
    }

    #[tokio::test]
    async fn test_delete_request() {
        let _m = mock("DELETE", "/v1/test_path/789")
            .with_status(204)
            .create();

        let client = create_test_client();
        let response: Response<serde_json::Value> =
            client.delete("test_path/789", None::<&()>).await.unwrap();

        assert_eq!(response.status, StatusCode::NO_CONTENT);
        assert!(response.body.is_null());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let _m = mock("GET", "/v1/test_path")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors": [{"code": "NOT_FOUND", "detail": "Resource not found"}]}"#)
            .create();

        let client = create_test_client();
        let result: Result<Response<serde_json::Value>, Error> =
            client.get("test_path", None::<&()>).await;

        let document = match result {
            Err(Error::Api(document)) => document,
            _ => panic!("Expected structured API error"),
        };
        assert_eq!(document.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            document.primary().and_then(|error| error.code()),
            Some("NOT_FOUND")
        );
    }

    #[tokio::test]
    async fn test_rate_limit_error() {
        let _m = mock("GET", "/v1/test_path")
            .with_status(429)
            .with_header("X-RateLimit-Window", "60")
            .with_header("Retry-After", "30")
            .with_header("X-RateLimit-Count", "100")
            .with_header("X-RateLimit-Limit", "100")
            .with_header("X-RateLimit-Remaining", "0")
            .with_header("X-RateLimit-Reset", "1620000000")
            .with_body(r#"{"errors": [{"code": "TOO_MANY_REQUESTS"}]}"#)
            .create();

        let client = create_test_client();
        let result: Result<Response<serde_json::Value>, Error> =
            client.get("test_path", None::<&()>).await;

        match result {
            Err(Error::RateLimitExceeded {
                window,
                count,
                limit,
                remaining,
                reset,
                retry_after,
            }) => {
                assert_eq!(window, "60");
                assert_eq!(count, 100);
                assert_eq!(limit, 100);
                assert_eq!(remaining, 0);
                assert_eq!(reset, 1620000000);
                assert_eq!(retry_after, 30);
            }
            _ => panic!("Expected RateLimitExceeded error"),
        }
    }
}
