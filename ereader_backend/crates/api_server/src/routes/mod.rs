//! API route handlers.

pub mod admin;
pub mod assets;
pub mod auth;
pub mod collections;
pub mod covers;
pub mod health;
pub mod library;
pub mod sync;

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{CorsLayer, Any};
use crate::middleware::auth_middleware;
use crate::state::AppState;

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    // Library routes with auth middleware
    let library_routes = Router::new()
        .route("/", get(library::list_books).post(library::create_book))
        .route("/search", get(library::search_books))
        .route(
            "/{id}",
            get(library::get_book)
                .put(library::update_book)
                .delete(library::delete_book),
        )
        .route("/{id}/upload", post(assets::upload_file))
        .route("/{id}/download", get(assets::download_file))
        .route("/{id}/download/{format}", get(assets::download_file_format))
        .route("/{id}/cover", get(covers::get_cover))
        .route("/{id}/cover/{size}", get(covers::get_cover_size))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB limit for file uploads
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Collections routes with auth middleware
    let collection_routes = Router::new()
        .route("/", get(collections::list_collections).post(collections::create_collection))
        .route(
            "/{id}",
            get(collections::get_collection)
                .put(collections::update_collection)
                .delete(collections::delete_collection),
        )
        .route("/{id}/books", post(collections::add_book))
        .route("/{id}/books/{book_id}", delete(collections::remove_book))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Sync routes with auth middleware
    let sync_routes = Router::new()
        .route("/", post(sync::sync_batch))
        .route("/reading-state/{book_id}", get(sync::get_reading_state).put(sync::update_reading_state))
        .route("/annotations", get(sync::list_annotations))
        .route("/annotations/{book_id}", get(sync::get_book_annotations))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Auth routes (device registration requires auth, webhook does not)
    let auth_routes = Router::new()
        .route("/device", post(auth::register_device)
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            )))
        .route("/webhook", post(auth::clerk_webhook)); // No auth middleware - validated via webhook secret

    // Admin routes with auth middleware
    let admin_routes = Router::new()
        .route("/reindex", post(admin::trigger_reindex))
        .route("/backup", post(admin::create_backup))
        .route("/stats", get(admin::get_stats))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let api_routes = Router::new()
        // Health endpoints (no auth)
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        // Auth endpoints
        .nest("/auth", auth_routes)
        // Library endpoints (with auth)
        .nest("/books", library_routes)
        // Collections endpoints (with auth)
        .nest("/collections", collection_routes)
        // Sync endpoints (with auth)
        .nest("/sync", sync_routes)
        // Admin endpoints (with auth)
        .nest("/admin", admin_routes);

    // Configure CORS for local development
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .nest("/api/v1", api_routes)
        .layer(cors)
        .with_state(state)
}
