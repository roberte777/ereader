//! User database queries.

use crate::models::{CreateUser, UpdateUser, User};
use crate::pool::DbPool;
use common::{Error, Result};

/// User-related database queries
pub struct UserQueries;

impl UserQueries {
    /// Get a user by ID
    pub async fn get_by_id(pool: &DbPool, id: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, name, created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Get a user by ID, returning an error if not found
    pub async fn get_by_id_required(pool: &DbPool, id: &str) -> Result<User> {
        Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| Error::not_found_resource("user", id))
    }

    /// Get a user by email
    pub async fn get_by_email(pool: &DbPool, email: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, name, created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// Create a new user or update if exists (upsert)
    pub async fn upsert(pool: &DbPool, data: &CreateUser) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, name)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE SET
                email = EXCLUDED.email,
                name = EXCLUDED.name
            RETURNING id, email, name, created_at, updated_at
            "#,
        )
        .bind(&data.id)
        .bind(&data.email)
        .bind(&data.name)
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Update an existing user
    pub async fn update(pool: &DbPool, id: &str, data: &UpdateUser) -> Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET
                email = COALESCE($2, email),
                name = COALESCE($3, name)
            WHERE id = $1
            RETURNING id, email, name, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&data.email)
        .bind(&data.name)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| Error::not_found_resource("user", id))?;

        Ok(user)
    }

    /// Delete a user
    pub async fn delete(pool: &DbPool, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List all users (for admin CLI)
    pub async fn list_all(pool: &DbPool) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, name, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}
