//! Reindex book task handler.

use crate::scheduler::TaskContext;
use crate::tasks::TaskHandler;
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

/// Payload for reindex book task
#[derive(Debug, Deserialize)]
struct ReindexPayload {
    book_id: Uuid,
}

/// Handler for reindexing book metadata
pub struct ReindexBookHandler;

#[async_trait]
impl TaskHandler for ReindexBookHandler {
    fn task_type(&self) -> &'static str {
        "reindex_book"
    }

    async fn execute(&self, ctx: &TaskContext, payload: &serde_json::Value) -> anyhow::Result<()> {
        let payload: ReindexPayload = serde_json::from_value(payload.clone())?;

        tracing::info!(book_id = %payload.book_id, "Reindexing book");

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

        // Extract metadata
        let metadata = handler.extract_metadata(&data)?;

        // Update book with new metadata
        let update = db_layer::models::UpdateBook {
            title: Some(metadata.title.unwrap_or_else(|| book.title.clone())),
            authors: if metadata.authors.is_empty() {
                None
            } else {
                Some(metadata.authors)
            },
            description: metadata.description.or(book.description.clone()),
            language: metadata.language.or(book.language.clone()),
            publisher: metadata.publisher.or(book.publisher.clone()),
            published_date: metadata.published_date.or(book.published_date.clone()),
            isbn: metadata.isbn.or(book.isbn.clone()),
            series_name: None,
            series_index: None,
            tags: None,
        };

        db_layer::queries::BookQueries::update_metadata(&ctx.pool, book.id, &update).await?;

        tracing::info!(book_id = %book.id, "Book reindexed successfully");

        Ok(())
    }
}
