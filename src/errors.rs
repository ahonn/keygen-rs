use reqwest::header::InvalidHeaderValue;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_urlencoded::ser::Error as UrlEncodedError;
use std::fmt;
use thiserror::Error;
use url::ParseError;

use crate::{license::License, license_file::LicenseFileDataset, machine_file::MachineFileDataset};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ApiErrorKind {
    InvalidCredentials,
    EmailRequired,
    EmailInvalid,
    PasswordRequired,
    PasswordInvalid,
    PasswordNotSupported,
    SsoRequired,
    OtpRequired,
    OtpInvalid,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApiErrorSource {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApiErrorObject {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source: Option<ApiErrorSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    links: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    meta: Option<Value>,
    #[serde(flatten)]
    extensions: Map<String, Value>,
}

impl ApiErrorObject {
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub fn source(&self) -> Option<&ApiErrorSource> {
        self.source.as_ref()
    }

    pub fn links(&self) -> Option<&Value> {
        self.links.as_ref()
    }

    pub fn meta(&self) -> Option<&Value> {
        self.meta.as_ref()
    }

    pub fn extensions(&self) -> &Map<String, Value> {
        &self.extensions
    }

    pub fn kind(&self) -> ApiErrorKind {
        match self.code() {
            Some("CREDENTIALS_INVALID") => ApiErrorKind::InvalidCredentials,
            Some("EMAIL_REQUIRED") => ApiErrorKind::EmailRequired,
            Some("EMAIL_INVALID") => ApiErrorKind::EmailInvalid,
            Some("PASSWORD_REQUIRED") => ApiErrorKind::PasswordRequired,
            Some("PASSWORD_INVALID") => ApiErrorKind::PasswordInvalid,
            Some("PASSWORD_NOT_SUPPORTED") => ApiErrorKind::PasswordNotSupported,
            Some("SSO_REQUIRED") => ApiErrorKind::SsoRequired,
            Some("OTP_REQUIRED") => ApiErrorKind::OtpRequired,
            Some("OTP_INVALID") => ApiErrorKind::OtpInvalid,
            _ => ApiErrorKind::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiErrorDocument {
    status: StatusCode,
    errors: Vec<ApiErrorObject>,
    body: Value,
}

impl ApiErrorDocument {
    pub(crate) fn from_response(status: StatusCode, body: Value) -> Self {
        #[derive(Deserialize)]
        struct ErrorEnvelope {
            #[serde(default)]
            errors: Vec<ApiErrorObject>,
        }

        let errors = serde_json::from_value::<ErrorEnvelope>(body.clone())
            .map(|envelope| envelope.errors)
            .unwrap_or_default();

        Self {
            status,
            errors,
            body,
        }
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn errors(&self) -> &[ApiErrorObject] {
        &self.errors
    }

    pub fn primary(&self) -> Option<&ApiErrorObject> {
        self.errors.first()
    }

    pub fn kind(&self) -> ApiErrorKind {
        self.primary()
            .map(ApiErrorObject::kind)
            .unwrap_or(ApiErrorKind::Unknown)
    }

    pub fn body(&self) -> &Value {
        &self.body
    }
}

impl fmt::Display for ApiErrorDocument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(error) = self.primary() {
            let code = error.code().unwrap_or("API_ERROR");
            let detail = error.detail().unwrap_or("unknown API error");
            write!(formatter, "Keygen API error {code}: {detail}")
        } else {
            write!(formatter, "Keygen API error with status {}", self.status)
        }
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("System clock is out of sync")]
    SystemClockUnsynced,

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] ParseError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    #[error("URL encoding error: {0}")]
    UrlEncode(#[from] UrlEncodedError),

    #[error("Unsupported Keygen API version: {0}")]
    UnsupportedApiVersion(String),

    #[error("{0}")]
    Api(ApiErrorDocument),

    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        window: String,
        count: u32,
        limit: u32,
        remaining: u32,
        reset: u64,
        retry_after: u64,
    },

    #[error("License key is missing")]
    LicenseKeyMissing,

    #[error("License scheme is missing")]
    LicenseSchemeMissing,

    #[error("License scheme is not supported")]
    LicenseSchemeNotSupported,

    #[error("License is not signed")]
    LicenseNotSigned,

    #[error("License key is not genuine")]
    LicenseKeyNotGenuine,

    #[error("Public key is missing")]
    PublicKeyMissing,

    #[error("Public key is invalid")]
    PublicKeyInvalid,

    #[error("License scheme unsupported")]
    LicenseSchemeUnsupported,

    #[error("{0}")]
    CertificateFileInvalid(String),

    #[error("{0}")]
    CertificateFileNotGenuine(String),

    #[error("{0}")]
    CertificateFileNotSupported(String),

    #[error("Certificate file expired")]
    CertificateFileExpired,

    #[error("License file invalid: {0}")]
    LicenseFileInvalid(String),

    #[error("License file not genuine: {0}")]
    LicenseFileNotGenuine(String),

    #[error("License file not supported: {0}")]
    LicenseFileNotSupported(String),

    #[error("License file not encrypted")]
    LicenseFileNotEncrypted,

    #[error("License file expired")]
    LicenseFileExpired(Box<LicenseFileDataset>),

    #[error("Machine file invalid")]
    MachineFileInvalid(String),

    #[error("Machine file not genuine")]
    MachineFileNotGenuine(String),

    #[error("Machine file not supported")]
    MachineFileNotSupported(String),

    #[error("License file expired")]
    MachineFileExpired(Box<MachineFileDataset>),

    #[error("API error: {detail}")]
    KeygenApiError {
        code: String,
        detail: String,
        body: serde_json::Value,
    },

    #[error("Token not allowed")]
    TokenNotAllowed { code: String, detail: String },

    #[error("Token format invalid")]
    TokenFormatInvalid { code: String, detail: String },

    #[error("Token invalid")]
    TokenInvalid { code: String, detail: String },

    #[error("Token expired")]
    TokenExpired { code: String, detail: String },

    #[error("License suspended")]
    LicenseSuspended { code: String, detail: String },

    #[error("License expired")]
    LicenseExpired { code: String, detail: String },

    #[error("License not allowed")]
    LicenseNotAllowed { code: String, detail: String },

    #[error("License not activated")]
    LicenseNotActivated {
        code: String,
        detail: String,
        license: Box<License>,
    },

    #[error("License key invalid")]
    LicenseKeyInvalid { code: String, detail: String },

    #[error("License token invalid")]
    LicenseTokenInvalid { code: String, detail: String },

    #[error("License has too many machines")]
    LicenseTooManyMachines { code: String, detail: String },

    #[error("License has too many cores")]
    LicenseTooManyCores { code: String, detail: String },

    #[error("License has too many processes")]
    LicenseTooManyProcesses { code: String, detail: String },

    #[error("Machine already activated")]
    MachineAlreadyActivated { code: String, detail: String },

    #[error("Machine limit exceeded")]
    MachineLimitExceeded { code: String, detail: String },

    #[error("Machine no longer exists")]
    MachineNotFound,

    #[error("Process limit exceeded")]
    ProcessLimitExceeded { code: String, detail: String },

    #[error("Process no longer exists")]
    ProcessNotFound,

    #[error("Component conflict")]
    ComponentConflict { code: String, detail: String },

    #[error("Component already activated")]
    ComponentAlreadyActivated { code: String, detail: String },

    #[error("Component is not activated")]
    ComponentNotActivated { code: String, detail: String },

    #[error("Environment error")]
    EnvironmentError { code: String, detail: String },

    #[error("Heartbeat dead")]
    HeartbeatDead { code: String, detail: String },

    #[error("Heartbeat ping failed")]
    HeartbeatPingFailed { code: String, detail: String },

    #[error("Heartbeat is required")]
    HeartbeatRequired { code: String, detail: String },

    #[error("Validation fingerprint scope is missing")]
    ValidationFingerprintMissing { code: String, detail: String },

    #[error("Validation components scope is missing")]
    ValidationComponentsMissing { code: String, detail: String },

    #[error("Validation product scope is missing")]
    ValidationProductMissing { code: String, detail: String },

    #[error("Not found")]
    NotFound { code: String, detail: String },

    #[error("Keygen signature validation failed: {reason}")]
    KeygenSignatureInvalid { reason: String },

    #[error("Keygen signature missing")]
    KeygenSignatureMissing,

    #[error("Configuration missing")]
    MissingConfiguration,
}

impl Error {
    pub fn api_error(&self) -> Option<&ApiErrorDocument> {
        match self {
            Self::Api(document) => Some(document),
            _ => None,
        }
    }

    pub fn is_api_code(&self, expected: &str) -> bool {
        self.api_error()
            .and_then(ApiErrorDocument::primary)
            .and_then(ApiErrorObject::code)
            == Some(expected)
    }
}

pub trait ErrorMeta {
    fn code(&self) -> String;
    fn detail(&self) -> String;
}

impl ErrorMeta for Error {
    fn code(&self) -> String {
        match self {
            Error::Api(document) => document
                .primary()
                .and_then(ApiErrorObject::code)
                .unwrap_or("API_ERROR")
                .to_string(),
            Error::KeygenApiError { code, .. }
            | Error::TokenNotAllowed { code, .. }
            | Error::TokenFormatInvalid { code, .. }
            | Error::TokenInvalid { code, .. }
            | Error::TokenExpired { code, .. }
            | Error::LicenseSuspended { code, .. }
            | Error::LicenseExpired { code, .. }
            | Error::LicenseNotAllowed { code, .. }
            | Error::LicenseNotActivated { code, .. }
            | Error::LicenseKeyInvalid { code, .. }
            | Error::LicenseTokenInvalid { code, .. }
            | Error::LicenseTooManyMachines { code, .. }
            | Error::LicenseTooManyCores { code, .. }
            | Error::LicenseTooManyProcesses { code, .. }
            | Error::MachineAlreadyActivated { code, .. }
            | Error::MachineLimitExceeded { code, .. }
            | Error::ProcessLimitExceeded { code, .. }
            | Error::ComponentConflict { code, .. }
            | Error::ComponentAlreadyActivated { code, .. }
            | Error::ComponentNotActivated { code, .. }
            | Error::EnvironmentError { code, .. }
            | Error::HeartbeatDead { code, .. }
            | Error::HeartbeatPingFailed { code, .. }
            | Error::HeartbeatRequired { code, .. }
            | Error::ValidationFingerprintMissing { code, .. }
            | Error::ValidationComponentsMissing { code, .. }
            | Error::ValidationProductMissing { code, .. }
            | Error::NotFound { code, .. } => code.to_string(),
            _ => "ERROR".to_string(),
        }
    }

    fn detail(&self) -> String {
        match self {
            Error::Api(document) => document
                .primary()
                .and_then(ApiErrorObject::detail)
                .map(str::to_owned)
                .unwrap_or_else(|| document.to_string()),
            Error::KeygenApiError { detail, .. }
            | Error::TokenNotAllowed { detail, .. }
            | Error::TokenFormatInvalid { detail, .. }
            | Error::TokenInvalid { detail, .. }
            | Error::TokenExpired { detail, .. }
            | Error::LicenseSuspended { detail, .. }
            | Error::LicenseExpired { detail, .. }
            | Error::LicenseNotAllowed { detail, .. }
            | Error::LicenseNotActivated { detail, .. }
            | Error::LicenseKeyInvalid { detail, .. }
            | Error::LicenseTokenInvalid { detail, .. }
            | Error::LicenseTooManyMachines { detail, .. }
            | Error::LicenseTooManyCores { detail, .. }
            | Error::LicenseTooManyProcesses { detail, .. }
            | Error::MachineAlreadyActivated { detail, .. }
            | Error::MachineLimitExceeded { detail, .. }
            | Error::ProcessLimitExceeded { detail, .. }
            | Error::ComponentConflict { detail, .. }
            | Error::ComponentAlreadyActivated { detail, .. }
            | Error::ComponentNotActivated { detail, .. }
            | Error::EnvironmentError { detail, .. }
            | Error::HeartbeatDead { detail, .. }
            | Error::HeartbeatPingFailed { detail, .. }
            | Error::HeartbeatRequired { detail, .. }
            | Error::ValidationFingerprintMissing { detail, .. }
            | Error::ValidationComponentsMissing { detail, .. }
            | Error::ValidationProductMissing { detail, .. }
            | Error::NotFound { detail, .. } => detail.to_string(),
            _ => self.to_string(),
        }
    }
}

#[cfg(test)]
mod api_error_tests {
    use super::*;
    use reqwest::StatusCode;
    use serde_json::json;

    #[test]
    fn classifies_v1_7_and_v1_8_credential_errors_without_losing_raw_codes() {
        let v1_7 = ApiErrorDocument::from_response(
            StatusCode::UNAUTHORIZED,
            json!({
                "errors": [{
                    "code": "CREDENTIALS_INVALID",
                    "detail": "email and password must be valid"
                }]
            }),
        );
        let v1_8 = ApiErrorDocument::from_response(
            StatusCode::UNAUTHORIZED,
            json!({
                "errors": [{
                    "code": "PASSWORD_REQUIRED",
                    "detail": "password is required",
                    "meta": { "attempt": 1 },
                    "futureField": true
                }]
            }),
        );

        assert_eq!(v1_7.kind(), ApiErrorKind::InvalidCredentials);
        assert_eq!(v1_8.kind(), ApiErrorKind::PasswordRequired);
        assert_eq!(
            v1_7.primary().and_then(|error| error.code()),
            Some("CREDENTIALS_INVALID")
        );
        assert_eq!(
            v1_8.primary().and_then(|error| error.code()),
            Some("PASSWORD_REQUIRED")
        );
        assert_eq!(v1_8.errors()[0].extensions()["futureField"], true);
    }

    #[test]
    fn preserves_multiple_and_unknown_api_errors() {
        let document = ApiErrorDocument::from_response(
            StatusCode::BAD_REQUEST,
            json!({
                "errors": [
                    { "code": "FUTURE_ERROR", "detail": "first" },
                    { "detail": "second" }
                ],
                "meta": { "requestId": "req-1" }
            }),
        );

        assert_eq!(document.kind(), ApiErrorKind::Unknown);
        assert_eq!(document.errors().len(), 2);
        assert_eq!(document.body()["meta"]["requestId"], "req-1");
    }
}
