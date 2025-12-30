//! Common types, configuration, and error handling for the e-reader API.
//!
//! This crate provides shared functionality used across all other crates in the workspace:
//!
//! - **Types**: Strongly-typed identifiers (BookId, UserId, etc.), content hashing, pagination
//! - **Error**: Unified error handling with HTTP response conversion
//! - **Config**: Application configuration with file and environment loading

pub mod config;
pub mod error;
pub mod types;

// Re-export commonly used items
pub use config::AppConfig;
pub use error::{Error, Result};
pub use types::{
    AnnotationId, AnnotationType, BookFormat, BookId, CollectionId, ContentHash, DeviceId,
    FileAssetId, Paginated, Pagination, ReadingLocation, UserId,
};
