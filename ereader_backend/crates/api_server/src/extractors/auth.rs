//! Authentication extractor.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

/// Authenticated user extracted from request
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: Option<String>,
}

impl AuthUser {
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            email: None,
        }
    }
}

/// Error when authentication fails
pub struct AuthError(String);

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let body = crate::error::ApiErrorResponse::new("unauthorized", self.0);
        (StatusCode::UNAUTHORIZED, Json(body)).into_response()
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Try to get user from extensions (set by auth middleware)
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or_else(|| AuthError("Not authenticated".to_string()))
    }
}
