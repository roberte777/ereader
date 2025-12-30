//! Cover database queries.

use crate::models::{Cover, CreateCover};
use crate::pool::DbPool;
use common::Result;
use uuid::Uuid;

/// Cover-related database queries
pub struct CoverQueries;

impl CoverQueries {
    /// Get a cover by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Cover>> {
        let cover = sqlx::query_as::<_, Cover>(
            r#"
            SELECT id, book_id, size_variant, width, height, storage_path, created_at
            FROM covers
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(cover)
    }

    /// Get all covers for a book
    pub async fn get_for_book(pool: &DbPool, book_id: Uuid) -> Result<Vec<Cover>> {
        let covers = sqlx::query_as::<_, Cover>(
            r#"
            SELECT id, book_id, size_variant, width, height, storage_path, created_at
            FROM covers
            WHERE book_id = $1
            ORDER BY width ASC
            "#,
        )
        .bind(book_id)
        .fetch_all(pool)
        .await?;

        Ok(covers)
    }

    /// Get a cover for a book with a specific size variant
    pub async fn get_for_book_size(
        pool: &DbPool,
        book_id: Uuid,
        size_variant: &str,
    ) -> Result<Option<Cover>> {
        let cover = sqlx::query_as::<_, Cover>(
            r#"
            SELECT id, book_id, size_variant, width, height, storage_path, created_at
            FROM covers
            WHERE book_id = $1 AND size_variant = $2
            "#,
        )
        .bind(book_id)
        .bind(size_variant)
        .fetch_optional(pool)
        .await?;

        Ok(cover)
    }

    /// Create a new cover (upsert - replaces existing for same book/size)
    pub async fn create(pool: &DbPool, data: &CreateCover) -> Result<Cover> {
        let cover = sqlx::query_as::<_, Cover>(
            r#"
            INSERT INTO covers (book_id, size_variant, width, height, storage_path)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (book_id, size_variant) DO UPDATE SET
                width = EXCLUDED.width,
                height = EXCLUDED.height,
                storage_path = EXCLUDED.storage_path
            RETURNING id, book_id, size_variant, width, height, storage_path, created_at
            "#,
        )
        .bind(data.book_id)
        .bind(&data.size_variant)
        .bind(data.width)
        .bind(data.height)
        .bind(&data.storage_path)
        .fetch_one(pool)
        .await?;

        Ok(cover)
    }

    /// Delete a cover
    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM covers WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all covers for a book
    pub async fn delete_for_book(pool: &DbPool, book_id: Uuid) -> Result<u64> {
        let result = sqlx::query("DELETE FROM covers WHERE book_id = $1")
            .bind(book_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected())
    }
}
