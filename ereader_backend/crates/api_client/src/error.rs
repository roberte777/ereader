//! API client error types.

use thiserror::Error;

/// Client-side error type
#[derive(Debug, Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("authentication error: {0}")]
    Authentication(String),

    #[error("authorization error: {0}")]
    Authorization(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("rate limited")]
    RateLimited,

    #[error("server error: {0}")]
    Server(String),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("offline queue error: {0}")]
    OfflineQueue(String),
}

impl Error {
    /// Check if this error is retryable (transient network issues, server errors)
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Network(_) | Error::RateLimited | Error::Server(_)
        )
    }

    /// Check if this is a client error (bad request, validation issues)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Error::Authentication(_)
                | Error::Authorization(_)
                | Error::Validation(_)
                | Error::NotFound(_)
                | Error::Conflict(_)
        )
    }

    /// Check if this error should trigger offline queue
    pub fn should_queue(&self) -> bool {
        matches!(self, Error::Network(_))
    }

    /// Create an error from HTTP status code
    pub fn from_status(status: reqwest::StatusCode, message: String) -> Self {
        match status.as_u16() {
            401 => Error::Authentication(message),
            403 => Error::Authorization(message),
            404 => Error::NotFound(message),
            409 => Error::Conflict(message),
            422 => Error::Validation(message),
            429 => Error::RateLimited,
            500..=599 => Error::Server(message),
            _ => Error::Server(format!("HTTP {}: {}", status, message)),
        }
    }
}

/// Result type alias for client operations
pub type Result<T> = std::result::Result<T, Error>;
