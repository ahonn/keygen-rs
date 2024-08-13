use reqwest::header::InvalidHeaderValue;
use serde_urlencoded::ser::Error as UrlEncodedError;
use thiserror::Error;
use url::ParseError;

use crate::license::License;

#[derive(Error, Debug)]
pub enum Error {
    // General errors
    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("Invalid URL")]
    InvalidUrl,

    #[error("System clock is out of sync")]
    SystemClockUnsynced,

    // HTTP and API related errors
    #[error("API error: {0}")]
    ApiError(serde_json::Value),

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

    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        window: String,
        count: u32,
        limit: u32,
        remaining: u32,
        reset: u64,
        retry_after: u64,
    },

    // Authentication and authorization errors
    #[error("Not authorized")]
    NotAuthorized,

    #[error("Token not allowed")]
    TokenNotAllowed,

    #[error("Token format invalid")]
    TokenFormatInvalid,

    #[error("Token invalid")]
    TokenInvalid,

    #[error("Token expired")]
    TokenExpired,

    // License related errors
    #[error("License not allowed")]
    LicenseNotAllowed,

    #[error("License not activated")]
    LicenseNotActivated(License),

    #[error("License suspended")]
    LicenseSuspended,

    #[error("License expired")]
    LicenseExpired,

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

    #[error("License key invalid")]
    LicenseKeyInvalid,

    #[error("License token invalid")]
    LicenseTokenInvalid,

    #[error("License has too many machines")]
    LicenseTooManyMachines,

    #[error("License has too many cores")]
    LicenseTooManyCores,

    #[error("License has too many processes")]
    LicenseTooManyProcesses,

    #[error("License scheme unsupported")]
    LicenseSchemeUnsupported,

    // Machine and component related errors
    #[error("Machine already activated")]
    MachineAlreadyActivated,

    #[error("Machine limit exceeded")]
    MachineLimitExceeded,

    #[error("Machine no longer exists")]
    MachineNotFound,

    #[error("Process limit exceeded")]
    ProcessLimitExceeded,

    #[error("Process no longer exists")]
    ProcessNotFound,

    #[error("Component conflict")]
    ComponentConflict,

    #[error("Component already activated")]
    ComponentAlreadyActivated,

    #[error("Component is not activated")]
    ComponentNotActivated,

    // Other specific errors
    #[error("Environment error")]
    EnvironmentError,

    #[error("Heartbeat dead")]
    HeartbeatDead,

    #[error("Heartbeat ping failed")]
    HeartbeatPingFailed,

    #[error("Heartbeat is required")]
    HeartbeatRequired,

    #[error("Public key is missing")]
    PublicKeyMissing,

    #[error("Public key is invalid")]
    PublicKeyInvalid,

    #[error("Validation fingerprint scope is missing")]
    ValidationFingerprintMissing,

    #[error("Validation components scope is missing")]
    ValidationComponentsMissing,

    #[error("Validation product scope is missing")]
    ValidationProductMissing,

    #[error("Not found")]
    NotFound,
}
