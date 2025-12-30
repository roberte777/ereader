//! API error handling.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// API error response structure for custom error formatting
#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorBody,
}

#[derive(Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
}

impl ApiErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ApiErrorBody {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "unauthorized", msg),
            ApiError::Internal(err) => {
                tracing::error!(error = %err, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Internal server error".to_string(),
                )
            }
        };

        let body = ApiErrorResponse::new(code, message);
        (status, Json(body)).into_response()
    }
}

/// Wrapper type for using Result<T, common::Error> in handlers
/// This enables the ? operator to work with common::Error in route handlers
pub struct AppError(pub common::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Delegate to common::Error's IntoResponse implementation
        self.0.into_response()
    }
}

impl From<common::Error> for AppError {
    fn from(err: common::Error) -> Self {
        Self(err)
    }
}

impl std::fmt::Debug for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppError({:?})", self.0)
    }
}
