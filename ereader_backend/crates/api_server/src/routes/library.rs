//! Library book endpoints.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use common::{types::Pagination, BookFormat};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::extractors::AuthUser;
use crate::state::AppState;
use db_layer::queries::{BookQueries, BookSortOptions, BookFilterOptions};

/// Query parameters for listing books
#[derive(Debug, Deserialize)]
pub struct ListBooksQuery {
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub series: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
}

/// Query parameters for searching books
#[derive(Debug, Deserialize)]
pub struct SearchBooksQuery {
    pub q: String,
    #[serde(default)]
    pub page: Option<i64>,
    #[serde(default)]
    pub per_page: Option<i64>,
}

/// Request body for creating a book
#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub isbn: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub published_date: Option<String>,
    #[serde(default)]
    pub series_name: Option<String>,
    #[serde(default)]
    pub series_index: Option<f32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Request body for updating a book
#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub authors: Option<Vec<String>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub isbn: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub published_date: Option<String>,
    #[serde(default)]
    pub series_name: Option<String>,
    #[serde(default)]
    pub series_index: Option<f32>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

/// Book response structure
#[derive(Debug, Serialize)]
pub struct BookResponse {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
    // File information
    pub format: Option<BookFormat>,
    pub file_size: Option<i64>,
    pub content_hash: Option<String>,
    pub has_file: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<db_layer::models::Book> for BookResponse {
    fn from(book: db_layer::models::Book) -> Self {
        let has_file = book.has_file();
        Self {
            id: book.id,
            title: book.title,
            authors: book.authors,
            description: book.description,
            language: book.language,
            publisher: book.publisher,
            published_date: book.published_date,
            isbn: book.isbn,
            series_name: book.series_name,
            series_index: book.series_index,
            tags: book.tags,
            format: book.format,
            file_size: book.file_size,
            content_hash: book.content_hash,
            has_file,
            created_at: book.created_at,
            updated_at: book.updated_at,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub has_more: bool,
}

/// List books for the authenticated user
pub async fn list_books(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<ListBooksQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let pagination = Pagination {
        limit: per_page,
        offset,
    };

    let sort = BookSortOptions {
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    };

    let filter = BookFilterOptions {
        tag: query.tag,
        series: query.series,
        author: query.author,
    };

    let books = BookQueries::list_for_user(
        &state.pool,
        &auth.user_id,
        &pagination,
        &sort,
        &filter,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to list books");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let response = PaginatedResponse {
        has_more: books.has_more(),
        items: books.items.into_iter().map(BookResponse::from).collect(),
        total: books.total,
        page,
        per_page,
    };

    Ok(Json(response))
}

/// Get a specific book by ID
pub async fn get_book(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let book = BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, book_id = %id, "Failed to get book");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(BookResponse::from(book)))
}

/// Create a new book
pub async fn create_book(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateBookRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let create_book = db_layer::models::CreateBook::new(&auth.user_id, &req.title)
        .with_authors(req.authors)
        .with_tags(req.tags);

    let mut book_data = create_book;
    book_data.description = req.description;
    book_data.isbn = req.isbn;
    book_data.publisher = req.publisher;
    book_data.language = req.language;
    book_data.published_date = req.published_date;
    book_data.series_name = req.series_name;
    book_data.series_index = req.series_index;

    let book = BookQueries::create(&state.pool, &book_data)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create book");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(book_id = %book.id, user_id = %auth.user_id, "Created book");

    Ok((StatusCode::CREATED, Json(BookResponse::from(book))))
}

/// Update an existing book
pub async fn update_book(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBookRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    // Verify ownership first
    let existing = BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, book_id = %id, "Failed to get book for update");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let update = db_layer::models::UpdateBook {
        title: req.title,
        authors: req.authors,
        description: req.description,
        isbn: req.isbn,
        publisher: req.publisher,
        language: req.language,
        published_date: req.published_date,
        series_name: req.series_name,
        series_index: req.series_index,
        tags: req.tags,
    };

    let book = BookQueries::update_metadata(&state.pool, existing.id, &update)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, book_id = %id, "Failed to update book");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    tracing::info!(book_id = %book.id, user_id = %auth.user_id, "Updated book");

    Ok(Json(BookResponse::from(book)))
}

/// Delete a book
pub async fn delete_book(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // Verify ownership first
    BookQueries::get_by_id_for_user(&state.pool, id, &auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, book_id = %id, "Failed to get book for delete");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let deleted = BookQueries::delete(&state.pool, id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, book_id = %id, "Failed to delete book");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        tracing::info!(book_id = %id, user_id = %auth.user_id, "Deleted book");
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Search books by title or author
pub async fn search_books(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(query): Query<SearchBooksQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    let pagination = Pagination {
        limit: per_page,
        offset,
    };

    let books = BookQueries::search(&state.pool, &auth.user_id, &query.q, &pagination)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, query = %query.q, "Failed to search books");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = PaginatedResponse {
        has_more: books.has_more(),
        items: books.items.into_iter().map(BookResponse::from).collect(),
        total: books.total,
        page,
        per_page,
    };

    Ok(Json(response))
}
