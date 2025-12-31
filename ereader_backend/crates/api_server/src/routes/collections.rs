//! Collection management endpoints.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use common::types::Pagination;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;

/// Request body for creating a collection
#[derive(Debug, Deserialize)]
pub struct CreateCollectionRequest {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_collection_type")]
    pub collection_type: String,
}

fn default_collection_type() -> String {
    "shelf".to_string()
}

/// Request body for updating a collection
#[derive(Debug, Deserialize)]
pub struct UpdateCollectionRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Request body for adding a book to a collection
#[derive(Debug, Deserialize)]
pub struct AddBookRequest {
    pub book_id: Uuid,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

/// Collection response structure
#[derive(Debug, Serialize)]
pub struct CollectionResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub collection_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Collection detail with book count
#[derive(Debug, Serialize)]
pub struct CollectionDetailResponse {
    #[serde(flatten)]
    pub collection: CollectionResponse,
    pub book_count: i64,
    pub books: Vec<CollectionBookResponse>,
}

#[derive(Debug, Serialize)]
pub struct CollectionBookResponse {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub added_at: chrono::DateTime<chrono::Utc>,
    pub sort_order: Option<i32>,
}

/// List all collections for the authenticated user
pub async fn list_collections(
    State(state): State<AppState>,
    user: AuthUser,
    Query(pagination): Query<Pagination>,
) -> Result<Json<common::types::Paginated<CollectionResponse>>, StatusCode> {
    let collections = db_layer::queries::CollectionQueries::list_for_user(
        &state.pool,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to list collections");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Apply pagination manually (TODO: implement in db_layer)
    let total = collections.len() as i64;
    let offset = pagination.offset;
    let limit = pagination.limit;
    let items: Vec<_> = collections
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .map(|c| CollectionResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            collection_type: c.collection_type,
            created_at: c.created_at,
            updated_at: c.updated_at,
        })
        .collect();

    Ok(Json(common::types::Paginated {
        items,
        total,
        limit,
        offset,
    }))
}

/// Create a new collection
pub async fn create_collection(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<CollectionResponse>), StatusCode> {
    let mut create_collection = db_layer::models::CreateCollection::new(user.user_id.clone(), req.name.clone())
        .with_type(req.collection_type.clone());

    if let Some(desc) = &req.description {
        create_collection = create_collection.with_description(desc.clone());
    }

    let collection = db_layer::queries::CollectionQueries::create(&state.pool, &create_collection)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create collection");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::CREATED,
        Json(CollectionResponse {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            collection_type: collection.collection_type,
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        }),
    ))
}

/// Get a collection by ID with its books
pub async fn get_collection(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<CollectionDetailResponse>, StatusCode> {
    let collection = db_layer::queries::CollectionQueries::get_by_id_for_user(
        &state.pool,
        id,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to get collection");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // TODO: Implement CollectionQueries::get_books in db_layer
    let books = vec![];

    Ok(Json(CollectionDetailResponse {
        collection: CollectionResponse {
            id: collection.id,
            name: collection.name,
            description: collection.description,
            collection_type: collection.collection_type,
            created_at: collection.created_at,
            updated_at: collection.updated_at,
        },
        book_count: books.len() as i64,
        books,
    }))
}

/// Update a collection
pub async fn update_collection(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCollectionRequest>,
) -> Result<Json<CollectionResponse>, StatusCode> {
    // Verify ownership first
    let _existing = db_layer::queries::CollectionQueries::get_by_id_for_user(
        &state.pool,
        id,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to get collection");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let update_data = db_layer::models::UpdateCollection {
        name: req.name,
        description: req.description,
    };

    let collection = db_layer::queries::CollectionQueries::update(&state.pool, id, &update_data)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to update collection");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(CollectionResponse {
        id: collection.id,
        name: collection.name,
        description: collection.description,
        collection_type: collection.collection_type,
        created_at: collection.created_at,
        updated_at: collection.updated_at,
    }))
}

/// Delete a collection
pub async fn delete_collection(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Verify ownership first
    let _existing = db_layer::queries::CollectionQueries::get_by_id_for_user(
        &state.pool,
        id,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to get collection");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let deleted = db_layer::queries::CollectionQueries::delete(&state.pool, id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to delete collection");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

/// Add a book to a collection
pub async fn add_book(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AddBookRequest>,
) -> Result<StatusCode, StatusCode> {
    // Verify collection ownership
    let _collection = db_layer::queries::CollectionQueries::get_by_id_for_user(
        &state.pool,
        id,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to get collection");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    // TODO: Verify book ownership with BookQueries::get_by_id_for_user

    // Add book to collection
    db_layer::queries::CollectionQueries::add_book(&state.pool, id, req.book_id, req.sort_order)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to add book to collection");
            // Could be a duplicate
            if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::CREATED)
}

/// Remove a book from a collection
pub async fn remove_book(
    State(state): State<AppState>,
    user: AuthUser,
    Path((collection_id, book_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    // Verify collection ownership
    let _collection = db_layer::queries::CollectionQueries::get_by_id_for_user(
        &state.pool,
        collection_id,
        &user.user_id,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to get collection");
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let deleted = db_layer::queries::CollectionQueries::remove_book(&state.pool, collection_id, book_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to remove book from collection");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}
