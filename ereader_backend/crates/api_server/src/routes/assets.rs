//! Asset endpoints for file uploads and downloads.

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::Response,
    Json,
};
use common::{BookFormat, ContentHash, Error, Result};
use db_layer::{models::CreateFileAsset, queries::{BookQueries, FileAssetQueries}};
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
            .ok_or_else(|| Error::Validation(format!("Unsupported format: {}", extension)))?;

        // Read file data
        let data = field
            .bytes()
            .await
            .map_err(|e| Error::Validation(e.to_string()))?;

        let content_hash = ContentHash::from_bytes(&data);
        let file_size = data.len() as i64;

        // Check for duplicate by hash (across user's library)
        let existing_assets = FileAssetQueries::find_by_hash(&state.pool, content_hash.as_str()).await?;
        for existing in existing_assets {
            // Check if this asset belongs to a book owned by the current user
            if let Some(_existing_book) = BookQueries::get_by_id_for_user(&state.pool, existing.book_id, &auth.user_id).await? {
                return Err(Error::Conflict(
                    "This file already exists in your library".into(),
                ));
            }
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

        // Create file asset record
        let _asset_id = Uuid::now_v7();
        let asset_data = CreateFileAsset::new(
            id,
            format,
            file_size,
            content_hash.as_str(),
            storage_path,
            filename,
        );

        let asset = FileAssetQueries::create(&state.pool, &asset_data).await?;

        tracing::info!(
            book_id = %id,
            asset_id = %asset.id,
            format = ?format,
            size_bytes = file_size,
            "File uploaded successfully"
        );

        return Ok(Json(UploadResponse {
            asset_id: asset.id,
            format,
            file_size,
            content_hash: content_hash.as_str().to_string(),
        }));
    }

    Err(Error::Validation("No file provided".into()))
}

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub asset_id: Uuid,
    pub format: BookFormat,
    pub file_size: i64,
    pub content_hash: String,
}

/// Download the default file for a book
pub async fn download_file(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    download_file_impl(state, auth, id, None).await
}

/// Download a specific format file for a book
pub async fn download_file_format(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((id, format)): Path<(Uuid, String)>,
) -> Result<Response> {
    let book_format = BookFormat::from_extension(&format)
        .ok_or_else(|| Error::Validation(format!("Unknown format: {}", format)))?;
    download_file_impl(state, auth, id, Some(book_format)).await
}

async fn download_file_impl(
    state: AppState,
    auth: AuthUser,
    id: Uuid,
    format: Option<BookFormat>,
) -> Result<Response> {
    // Verify ownership
    let _book = BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;

    // Get file asset
    let asset = if let Some(fmt) = format {
        FileAssetQueries::get_for_book_format(&state.pool, id, fmt)
            .await?
            .ok_or_else(|| Error::NotFound(format!("No {} file available", fmt)))?
    } else {
        FileAssetQueries::get_default_for_book(&state.pool, id)
            .await?
            .ok_or_else(|| Error::NotFound("No file available".into()))?
    };

    // Open file
    let path = state.storage.full_path(&asset.storage_path);
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, asset.format.mime_type())
        .header(header::CONTENT_LENGTH, asset.file_size)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", asset.original_filename),
        )
        .body(body)
        .map_err(|e| Error::Internal(e.to_string()))?;

    Ok(response)
}
