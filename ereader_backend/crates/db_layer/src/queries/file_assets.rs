//! File asset database queries.

use crate::models::{CreateFileAsset, FileAsset};
use crate::pool::DbPool;
use common::{BookFormat, Result};
use uuid::Uuid;

/// File asset-related database queries
pub struct FileAssetQueries;

impl FileAssetQueries {
    /// Get a file asset by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<FileAsset>> {
        let asset = sqlx::query_as::<_, FileAsset>(
            r#"
            SELECT id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            FROM file_assets
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(asset)
    }

    /// Get all file assets for a book
    pub async fn get_for_book(pool: &DbPool, book_id: Uuid) -> Result<Vec<FileAsset>> {
        let assets = sqlx::query_as::<_, FileAsset>(
            r#"
            SELECT id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            FROM file_assets
            WHERE book_id = $1
            ORDER BY created_at ASC
            "#,
        )
        .bind(book_id)
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    /// Get a file asset for a book with a specific format
    pub async fn get_for_book_format(
        pool: &DbPool,
        book_id: Uuid,
        format: BookFormat,
    ) -> Result<Option<FileAsset>> {
        let asset = sqlx::query_as::<_, FileAsset>(
            r#"
            SELECT id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            FROM file_assets
            WHERE book_id = $1 AND format = $2
            "#,
        )
        .bind(book_id)
        .bind(format)
        .fetch_optional(pool)
        .await?;

        Ok(asset)
    }

    /// Find file assets by content hash (for deduplication)
    pub async fn find_by_hash(pool: &DbPool, content_hash: &str) -> Result<Vec<FileAsset>> {
        let assets = sqlx::query_as::<_, FileAsset>(
            r#"
            SELECT id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            FROM file_assets
            WHERE content_hash = $1
            "#,
        )
        .bind(content_hash)
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }

    /// Create a new file asset
    pub async fn create(pool: &DbPool, data: &CreateFileAsset) -> Result<FileAsset> {
        let asset = sqlx::query_as::<_, FileAsset>(
            r#"
            INSERT INTO file_assets (id, book_id, format, file_size, content_hash, storage_path, original_filename)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (book_id, format) DO UPDATE SET
                file_size = EXCLUDED.file_size,
                content_hash = EXCLUDED.content_hash,
                storage_path = EXCLUDED.storage_path,
                original_filename = EXCLUDED.original_filename
            RETURNING id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            "#,
        )
        .bind(data.id)
        .bind(data.book_id)
        .bind(data.format)
        .bind(data.file_size)
        .bind(&data.content_hash)
        .bind(&data.storage_path)
        .bind(&data.original_filename)
        .fetch_one(pool)
        .await?;

        Ok(asset)
    }

    /// Delete a file asset
    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM file_assets WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all file assets for a book
    pub async fn delete_for_book(pool: &DbPool, book_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM file_assets WHERE book_id = $1")
            .bind(book_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Get the default (first) file asset for a book
    pub async fn get_default_for_book(pool: &DbPool, book_id: Uuid) -> Result<Option<FileAsset>> {
        let asset = sqlx::query_as::<_, FileAsset>(
            r#"
            SELECT id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            FROM file_assets
            WHERE book_id = $1
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .bind(book_id)
        .fetch_optional(pool)
        .await?;

        Ok(asset)
    }

    /// Get all file assets (for cleanup operations)
    pub async fn get_all(pool: &DbPool) -> Result<Vec<FileAsset>> {
        let assets = sqlx::query_as::<_, FileAsset>(
            r#"
            SELECT id, book_id, format, file_size, content_hash, storage_path, original_filename, created_at
            FROM file_assets
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(assets)
    }
}
