//! Database access layer for the e-reader API.
//!
//! This crate provides:
//! - Connection pool management
//! - Database models representing table rows
//! - Query functions organized by entity type
//! - Migration support via SQLx

pub mod models;
pub mod pool;
pub mod queries;

// Re-export commonly used items
pub use models::*;
pub use pool::{create_pool, create_pool_from_url, health_check, run_migrations, DbPool};
pub use queries::*;
