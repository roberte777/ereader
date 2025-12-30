//! Annotation database queries.

use crate::models::{Annotation, UpsertAnnotation};
use crate::pool::DbPool;
use chrono::{DateTime, Utc};
use common::{Error, Paginated, Pagination, Result};
use uuid::Uuid;

/// Annotation-related database queries
pub struct AnnotationQueries;

impl AnnotationQueries {
    /// Get an annotation by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Annotation>> {
        let annotation = sqlx::query_as::<_, Annotation>(
            r#"
            SELECT id, user_id, book_id, annotation_type, location_start, location_end,
                   content, style, created_at, updated_at, deleted_at
            FROM annotations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(annotation)
    }

    /// Get an annotation by ID, returning an error if not found
    pub async fn get_by_id_required(pool: &DbPool, id: Uuid) -> Result<Annotation> {
        Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| Error::not_found_resource("annotation", id))
    }

    /// List all annotations for a user (excluding deleted)
    pub async fn list_for_user(
        pool: &DbPool,
        user_id: &str,
        pagination: &Pagination,
    ) -> Result<Paginated<Annotation>> {
        // Get total count
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM annotations WHERE user_id = $1 AND deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        // Get paginated results
        let items = sqlx::query_as::<_, Annotation>(
            r#"
            SELECT id, user_id, book_id, annotation_type, location_start, location_end,
                   content, style, created_at, updated_at, deleted_at
            FROM annotations
            WHERE user_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(user_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(pool)
        .await?;

        Ok(Paginated::new(items, total, pagination))
    }

    /// Get all annotations for a specific book (excluding deleted)
    pub async fn get_for_book(
        pool: &DbPool,
        user_id: &str,
        book_id: Uuid,
    ) -> Result<Vec<Annotation>> {
        let annotations = sqlx::query_as::<_, Annotation>(
            r#"
            SELECT id, user_id, book_id, annotation_type, location_start, location_end,
                   content, style, created_at, updated_at, deleted_at
            FROM annotations
            WHERE user_id = $1 AND book_id = $2 AND deleted_at IS NULL
            ORDER BY location_start ASC
            "#,
        )
        .bind(user_id)
        .bind(book_id)
        .fetch_all(pool)
        .await?;

        Ok(annotations)
    }

    /// Get annotations updated since a specific time (for sync, includes deleted)
    pub async fn get_updated_since(
        pool: &DbPool,
        user_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Annotation>> {
        let annotations = sqlx::query_as::<_, Annotation>(
            r#"
            SELECT id, user_id, book_id, annotation_type, location_start, location_end,
                   content, style, created_at, updated_at, deleted_at
            FROM annotations
            WHERE user_id = $1 AND updated_at > $2
            ORDER BY updated_at ASC
            "#,
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(pool)
        .await?;

        Ok(annotations)
    }

    /// Create or update an annotation
    pub async fn upsert(pool: &DbPool, data: &UpsertAnnotation) -> Result<Annotation> {
        let annotation = sqlx::query_as::<_, Annotation>(
            r#"
            INSERT INTO annotations (id, user_id, book_id, annotation_type, location_start,
                                    location_end, content, style)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                annotation_type = EXCLUDED.annotation_type,
                location_start = EXCLUDED.location_start,
                location_end = EXCLUDED.location_end,
                content = EXCLUDED.content,
                style = EXCLUDED.style,
                deleted_at = NULL
            RETURNING id, user_id, book_id, annotation_type, location_start, location_end,
                      content, style, created_at, updated_at, deleted_at
            "#,
        )
        .bind(data.id)
        .bind(&data.user_id)
        .bind(data.book_id)
        .bind(data.annotation_type)
        .bind(&data.location_start)
        .bind(&data.location_end)
        .bind(&data.content)
        .bind(&data.style)
        .fetch_one(pool)
        .await?;

        Ok(annotation)
    }

    /// Soft delete an annotation
    pub async fn soft_delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE annotations SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Hard delete an annotation
    pub async fn hard_delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM annotations WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Restore a soft-deleted annotation
    pub async fn restore(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "UPDATE annotations SET deleted_at = NULL WHERE id = $1 AND deleted_at IS NOT NULL",
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
