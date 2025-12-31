//! Generate covers task handler.

use crate::scheduler::TaskContext;
use crate::tasks::TaskHandler;
use async_trait::async_trait;
use serde::Deserialize;
use storage_layer::traits::CoverStorage;
use uuid::Uuid;

/// Payload for generate covers task
#[derive(Debug, Deserialize)]
struct GenerateCoversPayload {
    book_id: Uuid,
}

/// Handler for generating cover images
pub struct GenerateCoversHandler;

#[async_trait]
impl TaskHandler for GenerateCoversHandler {
    fn task_type(&self) -> &'static str {
        "generate_covers"
    }

    async fn execute(&self, ctx: &TaskContext, payload: &serde_json::Value) -> anyhow::Result<()> {
        let payload: GenerateCoversPayload = serde_json::from_value(payload.clone())?;

        tracing::info!(book_id = %payload.book_id, "Generating covers for book");

        // Get the book
        let book = db_layer::queries::BookQueries::get_by_id(&ctx.pool, payload.book_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Book not found: {}", payload.book_id))?;

        // Check if book has a file
        let storage_path = match &book.storage_path {
            Some(path) => path,
            None => {
                tracing::warn!(book_id = %book.id, "No file found for book");
                return Ok(());
            }
        };

        let format = book.format
            .ok_or_else(|| anyhow::anyhow!("Book has storage_path but no format: {}", book.id))?;

        // Retrieve the file from storage
        let storage = ctx.storage.as_ref();
        let data = storage_layer::traits::Storage::retrieve(storage, storage_path).await?;

        // Get the format handler
        let handler = indexer::handler_for_format(format)
            .ok_or_else(|| anyhow::anyhow!("No handler for format: {:?}", format))?;

        // Extract cover
        let cover_data = match handler.extract_cover(&data) {
            Ok(Some(data)) => data,
            Ok(None) => {
                tracing::info!(book_id = %book.id, "No cover found in book");
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(book_id = %book.id, error = %e, "Failed to extract cover");
                return Ok(());
            }
        };

        // Store cover images at different sizes
        let cover_paths = storage.store_cover(book.id, &cover_data).await?;

        // Save cover paths to database
        for (size_variant, path) in [
            ("small", &cover_paths.small),
            ("medium", &cover_paths.medium),
            ("large", &cover_paths.large),
        ] {
            let (width, height) = match size_variant {
                "small" => (100, 150),
                "medium" => (200, 300),
                "large" => (400, 600),
                _ => (200, 300),
            };

            let cover = db_layer::models::CreateCover::new(
                book.id,
                size_variant,
                width,
                height,
                path,
            );

            db_layer::queries::CoverQueries::create(&ctx.pool, &cover).await?;
        }

        tracing::info!(book_id = %book.id, "Covers generated successfully");

        Ok(())
    }
}
