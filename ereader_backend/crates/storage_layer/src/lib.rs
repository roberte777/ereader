//! Storage layer for the e-reader API.
//!
//! This crate provides file storage abstraction:
//! - Content-addressable file storage using SHA-256 hashes
//! - Cover image storage with automatic resizing
//! - Local filesystem implementation (S3 can be added later)

pub mod local;
pub mod traits;

// Re-export commonly used items
pub use local::LocalStorage;
pub use traits::{CoverPaths, CoverSize, CoverStorage, Storage};
