//! API server for the e-reader backend.
//!
//! This crate provides:
//! - HTTP API server using Axum
//! - Authentication middleware
//! - Route handlers for books, sync, and health checks
//! - Application state management

pub mod error;
pub mod extractors;
pub mod jwt;
pub mod middleware;
pub mod routes;
pub mod state;

pub use error::ApiError;
pub use jwt::JwtValidator;
pub use routes::create_router;
pub use state::{AppState, AppStateConfig};
