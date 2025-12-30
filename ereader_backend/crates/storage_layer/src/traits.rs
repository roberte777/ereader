//! Storage trait definitions.

use async_trait::async_trait;
use common::{ContentHash, Result};
use std::path::PathBuf;

/// Trait for file storage operations
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store a file and return its storage path
    async fn store(&self, content_hash: &ContentHash, data: &[u8]) -> Result<String>;

    /// Retrieve file data by storage path
    async fn retrieve(&self, storage_path: &str) -> Result<Vec<u8>>;

    /// Check if a file exists
    async fn exists(&self, storage_path: &str) -> Result<bool>;

    /// Delete a file
    async fn delete(&self, storage_path: &str) -> Result<bool>;

    /// Get the full path to a file
    fn full_path(&self, storage_path: &str) -> PathBuf;

    /// Get the base storage path
    fn base_path(&self) -> &str;
}

/// Cover image size variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoverSize {
    Small,  // 100x150
    Medium, // 200x300
    Large,  // 400x600
}

impl CoverSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Self::Small => (100, 150),
            Self::Medium => (200, 300),
            Self::Large => (400, 600),
        }
    }

    pub fn all() -> [Self; 3] {
        [Self::Small, Self::Medium, Self::Large]
    }
}

impl std::fmt::Display for CoverSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Paths to generated cover files
#[derive(Debug, Clone)]
pub struct CoverPaths {
    pub small: String,
    pub medium: String,
    pub large: String,
}

impl CoverPaths {
    pub fn get(&self, size: CoverSize) -> &str {
        match size {
            CoverSize::Small => &self.small,
            CoverSize::Medium => &self.medium,
            CoverSize::Large => &self.large,
        }
    }
}

/// Trait for cover image storage operations
#[async_trait]
pub trait CoverStorage: Send + Sync {
    /// Store a cover image and generate all size variants
    ///
    /// Returns the paths to all generated sizes
    async fn store_cover(&self, book_id: uuid::Uuid, image_data: &[u8]) -> Result<CoverPaths>;

    /// Retrieve a cover image at a specific size
    async fn retrieve_cover(&self, book_id: uuid::Uuid, size: CoverSize) -> Result<Vec<u8>>;

    /// Check if a cover exists for a book
    async fn cover_exists(&self, book_id: uuid::Uuid) -> Result<bool>;

    /// Delete all cover variants for a book
    async fn delete_cover(&self, book_id: uuid::Uuid) -> Result<bool>;

    /// Get the storage path for a cover
    fn cover_path(&self, book_id: uuid::Uuid, size: CoverSize) -> String;
}
