use crate::config::get_config;
use crate::errors::Error;
use crate::verifier::Verifier;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use reqwest::{Client as ReqwestClient, Request, StatusCode};
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use std::time::Duration;
use url::Url;

pub struct Client {
    inner: ReqwestClient,
    options: ClientOptions,
}

#[derive(Clone)]
pub struct ClientOptions {
    pub account: String,
    pub environment: Option<String>,
    pub license_key: Option<String>,
    pub token: Option<String>,
    pub user_agent: Option<String>,
    pub api_url: String,
    pub api_version: String,
    pub api_prefix: String,
    pub verify_keygen_signature: bool,
}

#[derive(Debug)]
pub struct Response<T> {
    #[allow(dead_code)]
    pub status: StatusCode,
    #[allow(dead_code)]
    pub headers: HeaderMap,
    pub body: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorMeta {
    pub code: String,
    pub detail: String,
}

impl Client {
    pub fn default() -> Result<Self, Error> {
        let config = get_config()?;
        Self::new(ClientOptions {
            account: config.account.to_string(),
            environment: config.environment.clone(),
            #[cfg(feature = "license-key")]
            license_key: config.license_key.clone(),
            #[cfg(not(feature = "license-key"))]
            license_key: None,
            #[cfg(feature = "token")]
            token: config.token.clone(),
            #[cfg(not(feature = "token"))]
            token: None,
            user_agent: config.user_agent.clone(),
            api_url: config.api_url.to_string(),
            api_version: config.api_version.to_string(),
            api_prefix: config.api_prefix.to_string(),
            #[cfg(feature = "license-key")]
            verify_keygen_signature: config.verify_keygen_signature.unwrap_or(true),
            #[cfg(not(feature = "license-key"))]
            verify_keygen_signature: true,
        })
    }

    pub fn new(options: ClientOptions) -> Result<Self, Error> {
        let client = ReqwestClient::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| Error::UnexpectedError(format!("Failed to build HTTP client: {}", e)))?;

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

    fn new_request<T: Serialize + ?Sized>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<&T>,
    ) -> Result<Request, Error> {
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

        headers.insert(
            "Keygen-Version",
            HeaderValue::from_str(&self.options.api_version)?,
        );

        if let Some(key) = &self.options.license_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("License {}", key))?,
            );
        } else if let Some(token) = &self.options.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))?,
            );
        }

        let mut request = self.inner.request(method.clone(), url).headers(headers);

        if method != reqwest::Method::GET && params.is_some() {
            request = request.json(&json!(params));
        }
        Ok(request.build()?)
    }

    fn new_request_no_version<T: Serialize + ?Sized>(
        &self,
        method: reqwest::Method,
        path: &str,
        params: Option<&T>,
    ) -> Result<Request, Error> {
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

        // Note: Intentionally NOT setting Keygen-Version header for service introspection

        if let Some(key) = &self.options.license_key {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("License {}", key))?,
            );
        } else if let Some(token) = &self.options.token {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))?,
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
            (Some(h), Some(p)) => format!("{}:{}", h, p),
            (Some(h), None) => h.to_string(),
            _ => "api.keygen.sh".to_string(),
        };

        let response = self.inner.execute(request).await?;

        let status = response.status();
        let headers = response.headers().clone();

        if status.is_client_error() || status.is_server_error() {
            let error_body: serde_json::Value = response.json().await?;
            return Err(self.handle_error(status, &headers, error_body));
        }
        let bytes = response.bytes().await?;

        if self.options.verify_keygen_signature {
            let config = get_config()?;
            if let Some(public_key) = config.public_key {
                let verifier = Verifier::new(public_key);

                let base_path = url.path();
                let full_path = if let Some(query) = url.query() {
                    format!("{}?{}", base_path, query)
                } else {
                    base_path.to_string()
                };

                verifier
                    .verify_keygen_signature(&headers, &bytes, &method, &full_path, &host)
                    .map_err(|err| Error::KeygenSignatureInvalid {
                        reason: format!("Keygen signature validation failed: {}", err),
                    })?;
            }
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
            (Some(h), Some(p)) => format!("{}:{}", h, p),
            (Some(h), None) => h.to_string(),
            _ => "api.keygen.sh".to_string(),
        };

        let response = self.inner.execute(request).await?;

        let status = response.status();
        let headers = response.headers().clone();

        if status.is_client_error() || status.is_server_error() {
            let error_body: serde_json::Value = response.json().await?;
            return Err(self.handle_error(status, &headers, error_body));
        }

        let text = response.text().await?;

        if self.options.verify_keygen_signature {
            let config = get_config()?;
            if let Some(public_key) = config.public_key {
                let verifier = Verifier::new(public_key);

                let base_path = url.path();
                let full_path = if let Some(query) = url.query() {
                    format!("{}?{}", base_path, query)
                } else {
                    base_path.to_string()
                };

                verifier
                    .verify_keygen_signature(&headers, text.as_bytes(), &method, &full_path, &host)
                    .map_err(|err| Error::KeygenSignatureInvalid {
                        reason: format!("Keygen signature validation failed: {}", err),
                    })?;
            }
        }

        Ok(Response {
            status,
            headers,
            body: text,
        })
    }

    fn handle_error(
        &self,
        status: StatusCode,
        headers: &HeaderMap,
        body: serde_json::Value,
    ) -> Error {
        match status {
            StatusCode::TOO_MANY_REQUESTS => self.handle_rate_limit_error(headers),
            StatusCode::FORBIDDEN => self.handle_forbidden_error(&body),
            _ if status.is_server_error() => Error::UnexpectedError(format!(
                "Unexpected API error: status={}, body={}",
                status, body
            )),
            _ => self.handle_other_error(&body),
        }
    }

    fn handle_rate_limit_error(&self, headers: &HeaderMap) -> Error {
        // Handle rate limiting
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

    fn handle_forbidden_error(&self, body: &serde_json::Value) -> Error {
        let meta: Result<ErrorMeta, serde_json::Error> =
            serde_json::from_value(body["errors"][0].clone());
        if let Ok(meta) = meta {
            let detail = meta.detail.clone();
            let code = meta.code.clone();
            match code.as_str() {
                "TOKEN_NOT_ALLOWED" => Error::TokenNotAllowed { code, detail },
                "TOKEN_FORMAT_INVALID" => Error::TokenFormatInvalid { code, detail },
                "TOKEN_INVALID" => Error::TokenInvalid { code, detail },
                "TOKEN_EXPIRED" => Error::TokenExpired { code, detail },
                "LICENSE_NOT_ALLOWED" => Error::LicenseNotAllowed { code, detail },
                "LICENSE_SUSPENDED" => Error::LicenseSuspended { code, detail },
                "LICENSE_EXPIRED" => Error::LicenseExpired { code, detail },
                _ => Error::KeygenApiError {
                    code: code.clone(),
                    detail: detail.clone(),
                    body: body.clone(),
                },
            }
        } else {
            Error::KeygenApiError {
                code: "API_ERROR".to_string(),
                detail: "Unknown error".to_string(),
                body: body.clone(),
            }
        }
    }

    fn handle_other_error(&self, body: &serde_json::Value) -> Error {
        let meta: Result<ErrorMeta, serde_json::Error> =
            serde_json::from_value(body["errors"][0].clone());
        if let Ok(meta) = meta {
            let detail = meta.detail.clone();
            let code = meta.code.clone();
            match code.as_str() {
                "ENVIRONMENT_NOT_SUPPORTED" | "ENVIRONMENT_INVALID" => {
                    Error::EnvironmentError { code, detail }
                }
                "MACHINE_HEARTBEAT_DEAD" | "PROCESS_HEARTBEAT_DEAD" => {
                    Error::HeartbeatDead { code, detail }
                }
                "FINGERPRINT_TAKEN" => Error::MachineAlreadyActivated { code, detail },
                "MACHINE_LIMIT_EXCEEDED" => Error::MachineLimitExceeded { code, detail },
                "MACHINE_PROCESS_LIMIT_EXCEEDED" => Error::ProcessLimitExceeded { code, detail },
                "COMPONENTS_FINGERPRINT_CONFLICT" => Error::ComponentConflict { code, detail },
                "COMPONENTS_FINGERPRINT_TAKEN" => Error::ComponentAlreadyActivated { code, detail },
                "TOKEN_INVALID" => Error::LicenseTokenInvalid { code, detail },
                "LICENSE_INVALID" => Error::LicenseKeyInvalid { code, detail },
                "NOT_FOUND" => Error::NotFound { code, detail },
                _ => Error::KeygenApiError {
                    code: code.clone(),
                    detail: detail.clone(),
                    body: body.clone(),
                },
            }
        } else {
            Error::KeygenApiError {
                code: "API_ERROR".to_string(),
                detail: "Unknown error".to_string(),
                body: body.clone(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            api_version: "1.0".to_string(),
            api_prefix: "v1".to_string(),
            verify_keygen_signature: true, // Enable Keygen-Signature verification for tests
        }).expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_get_request() {
        let _m = mock("GET", "/v1/test_path")
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

        match result {
            Err(Error::NotFound { .. }) => {}
            _ => panic!("Expected NotFound error"),
        }
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
