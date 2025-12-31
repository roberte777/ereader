//! Book database queries.

use crate::models::{Book, CreateBook, UpdateBook};
use crate::pool::DbPool;
use common::{BookFormat, Error, Paginated, Pagination, Result};
use uuid::Uuid;

/// Sorting options for book list
#[derive(Debug, Clone, Default)]
pub struct BookSortOptions {
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl BookSortOptions {
    pub fn by_title() -> Self {
        Self {
            sort_by: Some("title".to_string()),
            sort_order: Some("asc".to_string()),
        }
    }

    pub fn by_created_at_desc() -> Self {
        Self {
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

/// Filter options for book list
#[derive(Debug, Clone, Default)]
pub struct BookFilterOptions {
    pub tag: Option<String>,
    pub series: Option<String>,
    pub author: Option<String>,
}

/// Book-related database queries
pub struct BookQueries;

impl BookQueries {
    /// List books for a user with pagination and filtering
    pub async fn list_for_user(
        pool: &DbPool,
        user_id: &str,
        pagination: &Pagination,
        sort: &BookSortOptions,
        filter: &BookFilterOptions,
    ) -> Result<Paginated<Book>> {
        // Build the WHERE clause dynamically using owned strings
        let mut where_clauses: Vec<String> = vec!["user_id = $1".to_string()];
        let mut param_idx = 2;

        if filter.tag.is_some() {
            where_clauses.push(format!("${}::text = ANY(tags)", param_idx));
            param_idx += 1;
        }
        if filter.series.is_some() {
            where_clauses.push(format!("series_name = ${}", param_idx));
            param_idx += 1;
        }
        if filter.author.is_some() {
            where_clauses.push(format!("${}::text = ANY(authors)", param_idx));
            param_idx += 1;
        }

        let where_clause = where_clauses.join(" AND ");

        // Determine sort column and order
        let sort_column = match sort.sort_by.as_deref() {
            Some("title") => "title",
            Some("created_at") => "created_at",
            Some("updated_at") => "updated_at",
            Some("series_index") => "series_index",
            _ => "created_at",
        };

        let sort_order = match sort.sort_order.as_deref() {
            Some("asc") => "ASC",
            Some("desc") => "DESC",
            _ => "DESC",
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) as count FROM books WHERE {}",
            where_clause
        );

        let mut count_builder = sqlx::query_scalar::<_, i64>(&count_query).bind(user_id);

        if let Some(tag) = &filter.tag {
            count_builder = count_builder.bind(tag);
        }
        if let Some(series) = &filter.series {
            count_builder = count_builder.bind(series);
        }
        if let Some(author) = &filter.author {
            count_builder = count_builder.bind(author);
        }

        let total = count_builder.fetch_one(pool).await?;

        // Get paginated results
        let query = format!(
            r#"
            SELECT id, user_id, title, authors, description, language, publisher,
                   published_date, isbn, series_name, series_index, tags,
                   format, content_hash, file_size, storage_path, original_filename,
                   created_at, updated_at
            FROM books
            WHERE {}
            ORDER BY {} {}
            LIMIT ${} OFFSET ${}
            "#,
            where_clause,
            sort_column,
            sort_order,
            param_idx,
            param_idx + 1
        );

        let mut query_builder = sqlx::query_as::<_, Book>(&query).bind(user_id);

        if let Some(tag) = &filter.tag {
            query_builder = query_builder.bind(tag);
        }
        if let Some(series) = &filter.series {
            query_builder = query_builder.bind(series);
        }
        if let Some(author) = &filter.author {
            query_builder = query_builder.bind(author);
        }

        let items = query_builder
            .bind(pagination.limit)
            .bind(pagination.offset)
            .fetch_all(pool)
            .await?;

        Ok(Paginated::new(items, total, pagination))
    }

    /// Get a book by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Book>> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, user_id, title, authors, description, language, publisher,
                   published_date, isbn, series_name, series_index, tags,
                   format, content_hash, file_size, storage_path, original_filename,
                   created_at, updated_at
            FROM books
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(book)
    }

    /// Get a book by ID, returning an error if not found
    pub async fn get_by_id_required(pool: &DbPool, id: Uuid) -> Result<Book> {
        Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| Error::not_found_resource("book", id))
    }

    /// Get a book by ID for a specific user (ensures ownership)
    pub async fn get_by_id_for_user(
        pool: &DbPool,
        id: Uuid,
        user_id: &str,
    ) -> Result<Option<Book>> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, user_id, title, authors, description, language, publisher,
                   published_date, isbn, series_name, series_index, tags,
                   format, content_hash, file_size, storage_path, original_filename,
                   created_at, updated_at
            FROM books
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(book)
    }

    /// Create a new book
    pub async fn create(pool: &DbPool, data: &CreateBook) -> Result<Book> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            INSERT INTO books (id, user_id, title, authors, description, language, publisher,
                              published_date, isbn, series_name, series_index, tags,
                              format, content_hash, file_size, storage_path, original_filename)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING id, user_id, title, authors, description, language, publisher,
                      published_date, isbn, series_name, series_index, tags,
                      format, content_hash, file_size, storage_path, original_filename,
                      created_at, updated_at
            "#,
        )
        .bind(data.id)
        .bind(&data.user_id)
        .bind(&data.title)
        .bind(&data.authors)
        .bind(&data.description)
        .bind(&data.language)
        .bind(&data.publisher)
        .bind(&data.published_date)
        .bind(&data.isbn)
        .bind(&data.series_name)
        .bind(data.series_index)
        .bind(&data.tags)
        .bind(data.format)
        .bind(&data.content_hash)
        .bind(data.file_size)
        .bind(&data.storage_path)
        .bind(&data.original_filename)
        .fetch_one(pool)
        .await?;

        Ok(book)
    }

    /// Update a book's metadata
    pub async fn update_metadata(pool: &DbPool, id: Uuid, data: &UpdateBook) -> Result<Book> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            UPDATE books
            SET
                title = COALESCE($2, title),
                authors = COALESCE($3, authors),
                description = COALESCE($4, description),
                language = COALESCE($5, language),
                publisher = COALESCE($6, publisher),
                published_date = COALESCE($7, published_date),
                isbn = COALESCE($8, isbn),
                series_name = COALESCE($9, series_name),
                series_index = COALESCE($10, series_index),
                tags = COALESCE($11, tags)
            WHERE id = $1
            RETURNING id, user_id, title, authors, description, language, publisher,
                      published_date, isbn, series_name, series_index, tags,
                      format, content_hash, file_size, storage_path, original_filename,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(&data.title)
        .bind(&data.authors)
        .bind(&data.description)
        .bind(&data.language)
        .bind(&data.publisher)
        .bind(&data.published_date)
        .bind(&data.isbn)
        .bind(&data.series_name)
        .bind(data.series_index)
        .bind(&data.tags)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| Error::not_found_resource("book", id))?;

        Ok(book)
    }

    /// Delete a book
    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM books WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Search books by title, author, or description
    pub async fn search(
        pool: &DbPool,
        user_id: &str,
        query: &str,
        pagination: &Pagination,
    ) -> Result<Paginated<Book>> {
        let search_pattern = format!("%{}%", query.to_lowercase());

        // Get total count
        let total = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM books
            WHERE user_id = $1
              AND (LOWER(title) LIKE $2
                   OR EXISTS (SELECT 1 FROM unnest(authors) a WHERE LOWER(a) LIKE $2)
                   OR LOWER(COALESCE(description, '')) LIKE $2)
            "#,
        )
        .bind(user_id)
        .bind(&search_pattern)
        .fetch_one(pool)
        .await?;

        // Get paginated results
        let items = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, user_id, title, authors, description, language, publisher,
                   published_date, isbn, series_name, series_index, tags,
                   format, content_hash, file_size, storage_path, original_filename,
                   created_at, updated_at
            FROM books
            WHERE user_id = $1
              AND (LOWER(title) LIKE $2
                   OR EXISTS (SELECT 1 FROM unnest(authors) a WHERE LOWER(a) LIKE $2)
                   OR LOWER(COALESCE(description, '')) LIKE $2)
            ORDER BY title ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(user_id)
        .bind(&search_pattern)
        .bind(pagination.limit)
        .bind(pagination.offset)
        .fetch_all(pool)
        .await?;

        Ok(Paginated::new(items, total, pagination))
    }

    /// Find a book by content hash for a specific user (for deduplication)
    pub async fn find_by_content_hash(
        pool: &DbPool,
        user_id: &str,
        content_hash: &str,
    ) -> Result<Option<Book>> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            SELECT id, user_id, title, authors, description, language, publisher,
                   published_date, isbn, series_name, series_index, tags,
                   format, content_hash, file_size, storage_path, original_filename,
                   created_at, updated_at
            FROM books
            WHERE user_id = $1 AND content_hash = $2
            "#,
        )
        .bind(user_id)
        .bind(content_hash)
        .fetch_optional(pool)
        .await?;

        Ok(book)
    }

    /// Update a book's file information
    pub async fn update_file(
        pool: &DbPool,
        id: Uuid,
        format: BookFormat,
        content_hash: &str,
        file_size: i64,
        storage_path: &str,
        original_filename: &str,
    ) -> Result<Book> {
        let book = sqlx::query_as::<_, Book>(
            r#"
            UPDATE books
            SET
                format = $2,
                content_hash = $3,
                file_size = $4,
                storage_path = $5,
                original_filename = $6
            WHERE id = $1
            RETURNING id, user_id, title, authors, description, language, publisher,
                      published_date, isbn, series_name, series_index, tags,
                      format, content_hash, file_size, storage_path, original_filename,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(format)
        .bind(content_hash)
        .bind(file_size)
        .bind(storage_path)
        .bind(original_filename)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| Error::not_found_resource("book", id))?;

        Ok(book)
    }
}
