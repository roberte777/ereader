//! Collection database queries.

use crate::models::{Book, Collection, CollectionBook, CreateCollection, UpdateCollection};
use crate::pool::DbPool;
use common::{Error, Paginated, Pagination, Result};
use uuid::Uuid;

/// Collection-related database queries
pub struct CollectionQueries;

impl CollectionQueries {
    /// List all collections for a user
    pub async fn list_for_user(pool: &DbPool, user_id: &str) -> Result<Vec<Collection>> {
        let collections = sqlx::query_as::<_, Collection>(
            r#"
            SELECT id, user_id, name, description, collection_type, created_at, updated_at
            FROM collections
            WHERE user_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(collections)
    }

    /// Get a collection by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Collection>> {
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            SELECT id, user_id, name, description, collection_type, created_at, updated_at
            FROM collections
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(collection)
    }

    /// Get a collection by ID, returning an error if not found
    pub async fn get_by_id_required(pool: &DbPool, id: Uuid) -> Result<Collection> {
        Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| Error::not_found_resource("collection", id))
    }

    /// Get a collection by ID for a specific user (ensures ownership)
    pub async fn get_by_id_for_user(
        pool: &DbPool,
        id: Uuid,
        user_id: &str,
    ) -> Result<Option<Collection>> {
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            SELECT id, user_id, name, description, collection_type, created_at, updated_at
            FROM collections
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(collection)
    }

    /// Create a new collection
    pub async fn create(pool: &DbPool, data: &CreateCollection) -> Result<Collection> {
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            INSERT INTO collections (id, user_id, name, description, collection_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, description, collection_type, created_at, updated_at
            "#,
        )
        .bind(data.id)
        .bind(&data.user_id)
        .bind(&data.name)
        .bind(&data.description)
        .bind(&data.collection_type)
        .fetch_one(pool)
        .await?;

        Ok(collection)
    }

    /// Update a collection
    pub async fn update(pool: &DbPool, id: Uuid, data: &UpdateCollection) -> Result<Collection> {
        let collection = sqlx::query_as::<_, Collection>(
            r#"
            UPDATE collections
            SET
                name = COALESCE($2, name),
                description = COALESCE($3, description)
            WHERE id = $1
            RETURNING id, user_id, name, description, collection_type, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&data.name)
        .bind(&data.description)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| Error::not_found_resource("collection", id))?;

        Ok(collection)
    }

    /// Delete a collection
    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM collections WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Add a book to a collection
    pub async fn add_book(
        pool: &DbPool,
        collection_id: Uuid,
        book_id: Uuid,
        sort_order: Option<i32>,
    ) -> Result<CollectionBook> {
        let membership = sqlx::query_as::<_, CollectionBook>(
            r#"
            INSERT INTO collection_books (collection_id, book_id, sort_order)
            VALUES ($1, $2, $3)
            ON CONFLICT (collection_id, book_id) DO UPDATE SET
                sort_order = COALESCE(EXCLUDED.sort_order, collection_books.sort_order)
            RETURNING collection_id, book_id, added_at, sort_order
            "#,
        )
        .bind(collection_id)
        .bind(book_id)
        .bind(sort_order)
        .fetch_one(pool)
        .await?;

        Ok(membership)
    }

    /// Remove a book from a collection
    pub async fn remove_book(pool: &DbPool, collection_id: Uuid, book_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM collection_books WHERE collection_id = $1 AND book_id = $2",
        )
        .bind(collection_id)
        .bind(book_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get all books in a collection
    pub async fn get_books(
        pool: &DbPool,
        collection_id: Uuid,
        pagination: &Pagination,
    ) -> Result<Paginated<Book>> {
        // Get total count
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM collection_books WHERE collection_id = $1",
        )
        .bind(collection_id)
        .fetch_one(pool)
        .await?;

        // Get paginated results
        let items = sqlx::query_as::<_, Book>(
            r#"
            SELECT b.id, b.user_id, b.title, b.authors, b.description, b.language, b.publisher,
                   b.published_date, b.isbn, b.series_name, b.series_index, b.tags, b.created_at, b.updated_at
            FROM books b
            JOIN collection_books cb ON b.id = cb.book_id
            WHERE cb.collection_id = $1
            ORDER BY COALESCE(cb.sort_order, 0), b.title
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(collection_id)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(pool)
        .await?;

        Ok(Paginated::new(items, total, pagination))
    }

    /// Check if a book is in a collection
    pub async fn contains_book(pool: &DbPool, collection_id: Uuid, book_id: Uuid) -> Result<bool> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM collection_books WHERE collection_id = $1 AND book_id = $2)",
        )
        .bind(collection_id)
        .bind(book_id)
        .fetch_one(pool)
        .await?;

        Ok(exists)
    }
}
