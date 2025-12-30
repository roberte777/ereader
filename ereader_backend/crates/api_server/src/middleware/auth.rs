//! Authentication middleware using Clerk JWT verification.

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::extractors::AuthUser;
use crate::state::AppState;

/// Middleware that validates JWT tokens and extracts user info
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get authorization header
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate JWT and extract claims using JWKS
    let claims = state
        .jwt_validator
        .validate(token)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "JWT validation failed");
            StatusCode::UNAUTHORIZED
        })?;

    // Extract user_id from claims
    let user_id = claims.id;

    if user_id.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Insert authenticated user into request extensions
    let auth_user = AuthUser::new(user_id);
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}
