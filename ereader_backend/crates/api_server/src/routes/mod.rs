//! API route handlers.

pub mod assets;
pub mod covers;
pub mod health;
pub mod library;

use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{get, post},
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

    let api_routes = Router::new()
        // Health endpoints (no auth)
        .route("/health", get(health::health_check))
        .route("/health/ready", get(health::readiness_check))
        // Library endpoints (with auth)
        .nest("/books", library_routes);

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
