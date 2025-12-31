//! Admin endpoints for maintenance and monitoring.

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::extractors::AuthUser;
use crate::state::AppState;

/// Statistics response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_users: i64,
    pub total_books: i64,
    pub total_devices: i64,
    pub total_collections: i64,
    pub total_annotations: i64,
    pub storage_used_bytes: i64,
    pub database_size_bytes: Option<i64>,
}

/// Reindex request
#[derive(Debug, Deserialize)]
pub struct ReindexRequest {
    #[serde(default)]
    pub book_ids: Option<Vec<uuid::Uuid>>,
    #[serde(default)]
    pub force: bool,
}

/// Reindex response
#[derive(Debug, Serialize)]
pub struct ReindexResponse {
    pub task_id: uuid::Uuid,
    pub status: String,
    pub message: String,
}

/// Backup request
#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    #[serde(default)]
    pub include_files: bool,
}

/// Backup response
#[derive(Debug, Serialize)]
pub struct BackupResponse {
    pub task_id: uuid::Uuid,
    pub status: String,
    pub message: String,
}

/// Trigger reindexing of books
/// This extracts metadata from uploaded files and regenerates covers
pub async fn trigger_reindex(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(req): Json<ReindexRequest>,
) -> Result<(StatusCode, Json<ReindexResponse>), StatusCode> {
    // TODO: Verify user has admin role
    // For now, we'll allow any authenticated user

    let payload = serde_json::json!({
        "book_ids": req.book_ids,
        "force": req.force,
    });

    let create_task = db_layer::models::CreateTask::new(
        if req.book_ids.is_some() {
            "reindex_specific_books"
        } else {
            "reindex_all_books"
        },
        payload,
    );

    let task = db_layer::queries::TaskQueries::create(&state.pool, &create_task)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create reindex task");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ReindexResponse {
            task_id: task.id,
            status: "queued".to_string(),
            message: if let Some(ref ids) = req.book_ids {
                format!("Reindex task created for {} books", ids.len())
            } else {
                "Reindex task created for all books".to_string()
            },
        }),
    ))
}

/// Create a backup
pub async fn create_backup(
    State(state): State<AppState>,
    _user: AuthUser,
    Json(req): Json<BackupRequest>,
) -> Result<(StatusCode, Json<BackupResponse>), StatusCode> {
    // TODO: Verify user has admin role

    let payload = serde_json::json!({
        "include_files": req.include_files,
        "created_at": chrono::Utc::now(),
    });

    let create_task = db_layer::models::CreateTask::new("create_backup", payload).with_priority(10);

    let task = db_layer::queries::TaskQueries::create(&state.pool, &create_task)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create backup task");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(BackupResponse {
            task_id: task.id,
            status: "queued".to_string(),
            message: if req.include_files {
                "Backup task created (database + files)".to_string()
            } else {
                "Backup task created (database only)".to_string()
            },
        }),
    ))
}

/// Get system statistics
pub async fn get_stats(
    State(_state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<StatsResponse>, StatusCode> {
    // TODO: Verify user has admin role
    // TODO: Implement StatsQueries in db_layer

    // For now, return placeholder statistics
    // These will be implemented in db_layer::queries::StatsQueries
    let total_users: i64 = 0;
    let total_books: i64 = 0;
    let total_devices: i64 = 0;
    let total_collections: i64 = 0;
    let total_annotations: i64 = 0;
    let storage_used_bytes: i64 = 0;
    let database_size_bytes: Option<i64> = None;

    Ok(Json(StatsResponse {
        total_users,
        total_books,
        total_devices,
        total_collections,
        total_annotations,
        storage_used_bytes,
        database_size_bytes,
    }))
}
