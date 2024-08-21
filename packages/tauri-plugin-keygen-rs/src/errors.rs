use serde::Serialize;
use thiserror::Error;
use keygen_rs::errors::{Error as KeygenError, ErrorMeta};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("Path resolve error: {0}")]
    PathResolveError(String),

    #[error("Keygen error: {0}")]
    KeygenError(#[from] KeygenError),
}

#[derive(Debug, Serialize)]
pub struct InvokeError {
    pub code: String,
    pub detail: String,
}

impl From<Error> for InvokeError {
    fn from(value: Error) -> Self {
        match value {
            Error::KeygenError(err ) => {
              Self {
                  code: err.code(),
                  detail: err.detail(),
              }
            }
            err => {
                let msg = match err {
                    Error::IoError(err) => err.to_string(),
                    Error::PathResolveError(msg) => msg,
                    _ => "Unknown error".into(),
                };
                Self {
                    code: "ERROR".into(),
                    detail: msg,
                }
            }
        }
    }
}
