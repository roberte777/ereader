//! Reading state database queries.

use crate::models::{ReadingState, UpsertReadingState};
use crate::pool::DbPool;
use chrono::{DateTime, Utc};
use common::Result;
use uuid::Uuid;

/// Reading state-related database queries
pub struct ReadingStateQueries;

impl ReadingStateQueries {
    /// Get reading state for a specific book and user
    pub async fn get_for_book(
        pool: &DbPool,
        user_id: &str,
        book_id: Uuid,
    ) -> Result<Option<ReadingState>> {
        let state = sqlx::query_as::<_, ReadingState>(
            r#"
            SELECT id, user_id, book_id, device_id, location, updated_at
            FROM reading_states
            WHERE user_id = $1 AND book_id = $2
            "#,
        )
        .bind(user_id)
        .bind(book_id)
        .fetch_optional(pool)
        .await?;

        Ok(state)
    }

    /// Get all reading states for a user
    pub async fn get_for_user(pool: &DbPool, user_id: &str) -> Result<Vec<ReadingState>> {
        let states = sqlx::query_as::<_, ReadingState>(
            r#"
            SELECT id, user_id, book_id, device_id, location, updated_at
            FROM reading_states
            WHERE user_id = $1
            ORDER BY updated_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(states)
    }

    /// Get reading states updated since a specific time (for sync)
    pub async fn get_updated_since(
        pool: &DbPool,
        user_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<ReadingState>> {
        let states = sqlx::query_as::<_, ReadingState>(
            r#"
            SELECT id, user_id, book_id, device_id, location, updated_at
            FROM reading_states
            WHERE user_id = $1 AND updated_at > $2
            ORDER BY updated_at ASC
            "#,
        )
        .bind(user_id)
        .bind(since)
        .fetch_all(pool)
        .await?;

        Ok(states)
    }

    /// Create or update a reading state
    pub async fn upsert(pool: &DbPool, data: &UpsertReadingState) -> Result<ReadingState> {
        let state = sqlx::query_as::<_, ReadingState>(
            r#"
            INSERT INTO reading_states (user_id, book_id, device_id, location)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id, book_id) DO UPDATE SET
                device_id = EXCLUDED.device_id,
                location = EXCLUDED.location,
                updated_at = NOW()
            RETURNING id, user_id, book_id, device_id, location, updated_at
            "#,
        )
        .bind(&data.user_id)
        .bind(data.book_id)
        .bind(data.device_id)
        .bind(sqlx::types::Json(&data.location))
        .fetch_one(pool)
        .await?;

        Ok(state)
    }

    /// Delete reading state for a book
    pub async fn delete(pool: &DbPool, user_id: &str, book_id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM reading_states WHERE user_id = $1 AND book_id = $2")
            .bind(user_id)
            .bind(book_id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
