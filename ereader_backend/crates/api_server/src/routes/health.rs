//! Health check endpoints.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub database: &'static str,
    pub storage: &'static str,
}

/// Basic health check - always returns OK if server is running
pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Readiness check - verifies database and storage connectivity
pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<ReadinessResponse>, StatusCode> {
    // Check database connection
    let db_healthy = db_layer::pool::health_check(&state.pool).await;

    // Check storage (verify base path exists)
    let storage_healthy = state.storage.health_check().await;

    if db_healthy && storage_healthy {
        Ok(Json(ReadinessResponse {
            status: "ready",
            database: "connected",
            storage: "available",
        }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
