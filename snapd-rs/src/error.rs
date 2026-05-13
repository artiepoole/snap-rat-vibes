use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    #[error("HTTP request error: {0}")]
    HttpRequest(#[from] hyper::http::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("connection error: {0}")]
    Connection(String),
    #[error("snapd error ({kind}): {message}")]
    Snapd { kind: String, message: String },
    #[error("unexpected response type: {0}")]
    UnexpectedResponseType(String),
}

impl Error {
    /// Returns true if this is a snapd error with the given kind string.
    pub fn is_kind(&self, kind: &str) -> bool {
        matches!(self, Error::Snapd { kind: k, .. } if k == kind)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
