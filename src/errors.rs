use reqwest::header::InvalidHeaderValue;
use serde_urlencoded::ser::Error as UrlEncodedError;
use thiserror::Error;
use url::ParseError;

use crate::{license::License, license_file::LicenseFileDataset, machine_file::MachineFileDataset};

#[derive(Error, Debug)]
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
    CerificateFileInvalid(String),

    #[error("{0}")]
    CertificateFileNotGenuine(String),

    #[error("{0}")]
    CertificateFileNotSupported(String),

    #[error("Cerificate file expired")]
    CerificateFileExpired,

    #[error("License file invalid: {0}")]
    LicenseFileInvalid(String),

    #[error("License file not genuine: {0}")]
    LicenseFileNotGenuine(String),

    #[error("License file not supported: {0}")]
    LicenseFileNotSupported(String),

    #[error("License file not encrypted")]
    LicenseFileNotEncrypted,

    #[error("License file expired")]
    LicenseFileExpired(LicenseFileDataset),

    #[error("Machine file invalid")]
    MachineFileInvalid(String),

    #[error("Machine file not genuine")]
    MachineFileNotGenuine(String),

    #[error("Machine file not supported")]
    MachineFileNotSupported(String),

    #[error("License file expired")]
    MachineFileExpired(MachineFileDataset),

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
        license: License,
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
}

pub trait ErrorMeta {
    fn code(&self) -> String;
    fn detail(&self) -> String;
}

impl ErrorMeta for Error {
    fn code(&self) -> String {
        match self {
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
