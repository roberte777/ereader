//! Error types and handling for the e-reader API.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Main error type for the application.
#[derive(Debug, Error)]
pub enum Error {
    // Authentication/Authorization
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    // Resource errors
    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    // Validation
    #[error("validation error: {0}")]
    Validation(String),

    // Storage
    #[error("storage error: {0}")]
    Storage(String),

    // Database
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    // External services
    #[error("external service error: {0}")]
    ExternalService(String),

    // Internal
    #[error("internal error: {0}")]
    Internal(String),

    // IO
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    // JSON
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// API error response body
#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl Error {
    /// Get the error code as a string
    pub fn code(&self) -> &'static str {
        match self {
            Error::Unauthorized(_) => "unauthorized",
            Error::Forbidden(_) => "forbidden",
            Error::NotFound(_) => "not_found",
            Error::Conflict(_) => "conflict",
            Error::Validation(_) => "validation_error",
            Error::Storage(_) => "storage_error",
            Error::Database(_) => "database_error",
            Error::ExternalService(_) => "external_service_error",
            Error::Internal(_) => "internal_error",
            Error::Io(_) => "io_error",
            Error::Json(_) => "json_error",
        }
    }

    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Error::Forbidden(_) => StatusCode::FORBIDDEN,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::Validation(_) => StatusCode::BAD_REQUEST,
            Error::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ExternalService(_) => StatusCode::BAD_GATEWAY,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Json(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// Check if this error should be logged at error level
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Error::Storage(_)
                | Error::Database(_)
                | Error::ExternalService(_)
                | Error::Internal(_)
                | Error::Io(_)
        )
    }

    /// Create a not found error for a specific resource type
    pub fn not_found_resource(resource: &str, id: impl std::fmt::Display) -> Self {
        Error::NotFound(format!("{} with id {} not found", resource, id))
    }

    /// Create a validation error with a specific field
    pub fn validation_field(field: &str, message: &str) -> Self {
        Error::Validation(format!("{}: {}", field, message))
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();

        let body = ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message: self.to_string(),
                details: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

/// Result type alias using our Error type
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(Error::Unauthorized("test".into()).code(), "unauthorized");
        assert_eq!(Error::NotFound("test".into()).code(), "not_found");
        assert_eq!(Error::Validation("test".into()).code(), "validation_error");
    }

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            Error::Unauthorized("test".into()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            Error::NotFound("test".into()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            Error::Validation("test".into()).status_code(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn test_not_found_resource() {
        let err = Error::not_found_resource("book", "123");
        assert!(err.to_string().contains("book"));
        assert!(err.to_string().contains("123"));
    }

    #[test]
    fn test_is_server_error() {
        assert!(Error::Internal("test".into()).is_server_error());
        assert!(Error::Storage("test".into()).is_server_error());
        assert!(!Error::NotFound("test".into()).is_server_error());
        assert!(!Error::Validation("test".into()).is_server_error());
    }
}
