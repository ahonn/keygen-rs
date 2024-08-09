use reqwest::header::InvalidHeaderValue;
use serde_urlencoded::ser::Error as UrlEncodedError;
use thiserror::Error;
use url::ParseError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("API error: {0}")]
    ApiError(serde_json::Value),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("URL parse error: {0}")]
    UrlParse(#[from] ParseError),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    #[error("URL encoding error: {0}")]
    UrlEncode(#[from] UrlEncodedError),

    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        window: String,
        count: u32,
        limit: u32,
        remaining: u32,
        reset: u64,
        retry_after: u64,
    },

    #[error("Token not allowed")]
    TokenNotAllowed,

    #[error("Token format invalid")]
    TokenFormatInvalid,

    #[error("Token invalid")]
    TokenInvalid,

    #[error("Token expired")]
    TokenExpired,

    #[error("License not allowed")]
    LicenseNotAllowed,

    #[error("License suspended")]
    LicenseSuspended,

    #[error("License expired")]
    LicenseExpired,

    #[error("Not authorized")]
    NotAuthorized,

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("Environment error")]
    EnvironmentError,

    #[error("Heartbeat dead")]
    HeartbeatDead,

    #[error("Machine already activated")]
    MachineAlreadyActivated,

    #[error("Machine limit exceeded")]
    MachineLimitExceeded,

    #[error("Process limit exceeded")]
    ProcessLimitExceeded,

    #[error("Component conflict")]
    ComponentConflict,

    #[error("Component already activated")]
    ComponentAlreadyActivated,

    #[error("License token invalid")]
    LicenseTokenInvalid,

    #[error("License key invalid")]
    LicenseKeyInvalid,

    #[error("Not found")]
    NotFound,
}
