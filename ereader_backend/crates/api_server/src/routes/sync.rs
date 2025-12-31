//! Sync endpoints - STUB IMPLEMENTATION
//! These are placeholder implementations until ReadingStateQueries and AnnotationQueries
//! are implemented in db_layer.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use common::types::ReadingLocation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingStateSync {
    pub book_id: Uuid,
    pub location: ReadingLocation,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationSync {
    #[serde(default)]
    pub id: Option<Uuid>,
    pub book_id: Uuid,
    pub annotation_type: AnnotationType,
    pub location_start: String,
    #[serde(default)]
    pub location_end: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
}

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub device_id: Uuid,
    #[serde(default)]
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub reading_states: Vec<ReadingStateSync>,
    #[serde(default)]
    pub annotations: Vec<AnnotationSync>,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub server_time: chrono::DateTime<chrono::Utc>,
    pub reading_states: Vec<ReadingStateSync>,
    pub annotations: Vec<AnnotationSync>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Serialize)]
pub struct SyncConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub local_updated_at: chrono::DateTime<chrono::Utc>,
    pub server_updated_at: chrono::DateTime<chrono::Utc>,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Merged,
}

#[derive(Debug, Serialize)]
pub struct ReadingStateResponse {
    pub book_id: Uuid,
    pub location: ReadingLocation,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReadingStateRequest {
    pub device_id: Uuid,
    pub location: ReadingLocation,
}

#[derive(Debug, Serialize)]
pub struct AnnotationResponse {
    pub id: Uuid,
    pub book_id: Uuid,
    pub annotation_type: AnnotationType,
    pub location_start: String,
    pub location_end: Option<String>,
    pub content: Option<String>,
    pub style: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Perform batch sync - STUB
pub async fn sync_batch(
    State(_state): State<AppState>,
    _user: AuthUser,
    Json(_request): Json<SyncRequest>,
) -> Result<Json<SyncResponse>, StatusCode> {
    // TODO: Implement with ReadingStateQueries and AnnotationQueries
    Ok(Json(SyncResponse {
        server_time: chrono::Utc::now(),
        reading_states: vec![],
        annotations: vec![],
        conflicts: vec![],
    }))
}

/// Get reading state - STUB
pub async fn get_reading_state(
    State(_state): State<AppState>,
    _user: AuthUser,
    Path(_book_id): Path<Uuid>,
) -> Result<Json<Option<ReadingStateResponse>>, StatusCode> {
    // TODO: Implement with ReadingStateQueries
    Ok(Json(None))
}

/// Update reading state - STUB
pub async fn update_reading_state(
    State(_state): State<AppState>,
    _user: AuthUser,
    Path(book_id): Path<Uuid>,
    Json(req): Json<UpdateReadingStateRequest>,
) -> Result<Json<ReadingStateResponse>, StatusCode> {
    // TODO: Implement with ReadingStateQueries
    Ok(Json(ReadingStateResponse {
        book_id,
        location: req.location,
        updated_at: chrono::Utc::now(),
    }))
}

/// List annotations - STUB
pub async fn list_annotations(
    State(_state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<AnnotationResponse>>, StatusCode> {
    // TODO: Implement with AnnotationQueries
    Ok(Json(vec![]))
}

/// Get book annotations - STUB
pub async fn get_book_annotations(
    State(_state): State<AppState>,
    _user: AuthUser,
    Path(_book_id): Path<Uuid>,
) -> Result<Json<Vec<AnnotationResponse>>, StatusCode> {
    // TODO: Implement with AnnotationQueries
    Ok(Json(vec![]))
}
