//! Cover image endpoints.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use common::{Error, Result};
use db_layer::queries::BookQueries;
use storage_layer::{CoverSize, CoverStorage};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;

/// Get a cover image (default: medium size)
pub async fn get_cover(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    get_cover_impl(state, auth, id, CoverSize::Medium).await
}

/// Get a cover image at a specific size
pub async fn get_cover_size(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, size)): Path<(Uuid, String)>,
) -> Result<Response> {
    let cover_size = match size.as_str() {
        "small" => CoverSize::Small,
        "medium" => CoverSize::Medium,
        "large" => CoverSize::Large,
        _ => return Err(Error::Validation(format!("Invalid cover size: {}. Must be small, medium, or large", size))),
    };

    get_cover_impl(state, auth, id, cover_size).await
}

async fn get_cover_impl(
    state: AppState,
    auth: AuthUser,
    id: Uuid,
    size: CoverSize,
) -> Result<Response> {
    // Verify book exists and belongs to user
    let _book = BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;

    // Check if cover exists
    if !state.storage.cover_exists(id).await? {
        return Err(Error::NotFound("Cover not found for this book".into()));
    }

    // Get the cover path
    let cover_path = state.storage.cover_path(id, size);
    let full_path = state.storage.covers_path().join(&cover_path);

    // Open and stream the cover file
    let file = tokio::fs::File::open(&full_path)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, path = ?full_path, "Failed to open cover file");
            Error::Storage(format!("Failed to open cover file: {}", e))
        })?;

    let metadata = file.metadata().await.map_err(|e| {
        Error::Storage(format!("Failed to read file metadata: {}", e))
    })?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CONTENT_LENGTH, metadata.len())
        .header(header::CACHE_CONTROL, "public, max-age=31536000") // Cache for 1 year
        .body(body)
        .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(response)
}
