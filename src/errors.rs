use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("License is invalid")]
    LicenseInvalid,

    #[error("License is not activated")]
    LicenseNotActivated,

    #[error("License is expired")]
    LicenseExpired,

    #[error("Machine limit exceeded")]
    MachineLimitExceeded,

    #[error("Process limit exceeded")]
    ProcessLimitExceeded,

    #[error("Component not activated")]
    ComponentNotActivated,

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    // Add more error types as needed
}
