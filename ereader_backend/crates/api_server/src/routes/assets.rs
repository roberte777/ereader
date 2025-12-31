//! Asset endpoints for file uploads and downloads.

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use common::{BookFormat, ContentHash, Error, Result};
use db_layer::queries::BookQueries;
use serde::Serialize;
use storage_layer::{CoverStorage, Storage};
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;

/// Upload a file for a book
pub async fn upload_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    // Verify book exists and belongs to user
    let _book = BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to parse multipart field");
            Error::Validation(format!("Error parsing multipart data: {}", e))
        })?
    {
        let filename = field
            .file_name()
            .ok_or_else(|| Error::Validation("Missing filename in upload".into()))?
            .to_string();

        tracing::debug!(filename = %filename, "Processing uploaded file");

        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Validation("Unknown file extension".into()))?;

        let format = BookFormat::from_extension(extension)
            .ok_or_else(|| Error::Validation(format!("Unsupported format: {}. Only EPUB is supported.", extension)))?;

        // Read file data
        let data = field
            .bytes()
            .await
            .map_err(|e| Error::Validation(e.to_string()))?;

        let content_hash = ContentHash::from_bytes(&data);
        let file_size = data.len() as i64;

        // Check for duplicate by hash (across user's library)
        if let Some(_existing_book) = BookQueries::find_by_content_hash(&state.pool, &auth.user_id, content_hash.as_str()).await? {
            return Err(Error::Conflict(
                "This file already exists in your library".into(),
            ));
        }

        // Store file
        let storage_path = state.storage.store(&content_hash, &data).await?;

        // Extract metadata and update book if available
        if let Ok(metadata) = indexer::extract_metadata(format, &data) {
            if metadata.has_data() {
                // Update book metadata if we have meaningful data
                let update = db_layer::models::UpdateBook {
                    title: metadata.title,
                    authors: if metadata.authors.is_empty() {
                        None
                    } else {
                        Some(metadata.authors)
                    },
                    description: metadata.description,
                    language: metadata.language,
                    publisher: metadata.publisher,
                    published_date: metadata.published_date,
                    isbn: metadata.isbn,
                    series_name: metadata.series_name,
                    series_index: metadata.series_index,
                    tags: None,
                };

                let _ = BookQueries::update_metadata(&state.pool, id, &update).await;
            }
        }

        // Extract and store cover if present
        if let Ok(Some(cover_data)) = indexer::extract_cover(format, &data) {
            // Delete existing covers first
            if let Ok(true) = state.storage.cover_exists(id).await {
                let _ = state.storage.delete_cover(id).await;
            }

            // Store new cover
            if let Ok(_cover_paths) = state.storage.store_cover(id, &cover_data).await {
                tracing::info!(book_id = %id, "Stored cover image");
            }
        }

        // Update book with file information
        let book = BookQueries::update_file(
            &state.pool,
            id,
            format,
            content_hash.as_str(),
            file_size,
            &storage_path,
            &filename,
        ).await?;

        tracing::info!(
            book_id = %id,
            format = ?format,
            size_bytes = file_size,
            "File uploaded successfully"
        );

        return Ok(Json(UploadResponse {
            book_id: book.id,
            format,
            file_size,
            content_hash: content_hash.as_str().to_string(),
        }));
    }

    Err(Error::Validation("No file provided".into()))
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub book_id: Uuid,
    pub format: BookFormat,
    pub file_size: i64,
    pub content_hash: String,
}

/// Download the file for a book
pub async fn download_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    // Verify ownership and get book
    let book = BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;

    // Check if book has a file
    let storage_path = book.storage_path
        .ok_or_else(|| Error::NotFound("No file available for this book".into()))?;
    let format = book.format
        .ok_or_else(|| Error::NotFound("No file available for this book".into()))?;
    let file_size = book.file_size
        .ok_or_else(|| Error::NotFound("No file available for this book".into()))?;
    let original_filename = book.original_filename
        .unwrap_or_else(|| format!("{}.{}", book.title, format.extension()));

    // Open file
    let path = state.storage.full_path(&storage_path);
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, format.mime_type())
        .header(header::CONTENT_LENGTH, file_size)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", original_filename),
        )
        .body(body)
        .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(response)
}
