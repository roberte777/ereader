//! Core types shared across all crates.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Unique identifier for books (UUID v7 for time-ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct BookId(pub Uuid);

impl BookId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for BookId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BookId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for BookId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for users (Clerk user ID is a string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for UserId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for UserId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Unique identifier for devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct DeviceId(pub Uuid);

impl DeviceId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for DeviceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for DeviceId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for collections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct CollectionId(pub Uuid);

impl CollectionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for CollectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CollectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for CollectionId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Unique identifier for annotations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct AnnotationId(pub Uuid);

impl AnnotationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for AnnotationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AnnotationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for AnnotationId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Content hash for deduplication and integrity (SHA-256 hex string)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct ContentHash(pub String);

impl ContentHash {
    /// Create a content hash from raw bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hex::encode(hasher.finalize()))
    }

    /// Create a content hash from a hex string
    pub fn from_hex(hex_str: impl Into<String>) -> Self {
        Self(hex_str.into())
    }

    /// Get the hex string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the first few characters for use in storage paths
    pub fn prefix(&self, len: usize) -> &str {
        &self.0[..len.min(self.0.len())]
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reading position - format-agnostic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingLocation {
    /// EPUB CFI, PDF page number, or generic offset
    pub locator: String,
    /// Estimated progress 0.0-1.0
    pub progress: f32,
    /// Optional chapter/section info
    pub chapter: Option<String>,
}

impl ReadingLocation {
    pub fn new(locator: impl Into<String>, progress: f32) -> Self {
        Self {
            locator: locator.into(),
            progress: progress.clamp(0.0, 1.0),
            chapter: None,
        }
    }

    pub fn with_chapter(mut self, chapter: impl Into<String>) -> Self {
        self.chapter = Some(chapter.into());
        self
    }
}

/// Supported ebook formats
///
/// Currently only EPUB is supported. To add new formats in the future:
/// 1. Add variant here
/// 2. Update from_extension(), mime_type(), extension()
/// 3. Create handler in indexer crate
/// 4. Run database migration to add enum value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "book_format", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum BookFormat {
    Epub,
}

impl BookFormat {
    /// Determine format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "epub" => Some(Self::Epub),
            _ => None,
        }
    }

    /// Get the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Epub => "application/epub+zip",
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Epub => "epub",
        }
    }
}

impl std::fmt::Display for BookFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Epub => write!(f, "epub"),
        }
    }
}

/// Annotation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "annotation_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
}

impl std::fmt::Display for AnnotationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Highlight => write!(f, "highlight"),
            Self::Note => write!(f, "note"),
            Self::Bookmark => write!(f, "bookmark"),
        }
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            offset: 0,
        }
    }
}

impl Pagination {
    pub fn new(limit: i64, offset: i64) -> Self {
        Self { limit, offset }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, total: i64, pagination: &Pagination) -> Self {
        Self {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        }
    }

    pub fn has_more(&self) -> bool {
        self.offset + (self.items.len() as i64) < self.total
    }

    pub fn next_offset(&self) -> i64 {
        self.offset + (self.items.len() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_hash() {
        let data = b"hello world";
        let hash = ContentHash::from_bytes(data);
        assert_eq!(hash.as_str().len(), 64); // SHA-256 produces 64 hex chars
        assert_eq!(hash.prefix(4), &hash.0[..4]);
    }

    #[test]
    fn test_book_format_from_extension() {
        assert_eq!(BookFormat::from_extension("epub"), Some(BookFormat::Epub));
        assert_eq!(BookFormat::from_extension("EPUB"), Some(BookFormat::Epub));
        // Unsupported formats return None
        assert_eq!(BookFormat::from_extension("pdf"), None);
        assert_eq!(BookFormat::from_extension("cbz"), None);
        assert_eq!(BookFormat::from_extension("mobi"), None);
        assert_eq!(BookFormat::from_extension("txt"), None);
    }

    #[test]
    fn test_reading_location() {
        let loc = ReadingLocation::new("page:42", 0.5).with_chapter("Chapter 5");
        assert_eq!(loc.locator, "page:42");
        assert_eq!(loc.progress, 0.5);
        assert_eq!(loc.chapter, Some("Chapter 5".to_string()));
    }

    #[test]
    fn test_reading_location_clamps_progress() {
        let loc = ReadingLocation::new("page:1", 1.5);
        assert_eq!(loc.progress, 1.0);

        let loc = ReadingLocation::new("page:1", -0.5);
        assert_eq!(loc.progress, 0.0);
    }

    #[test]
    fn test_pagination_defaults() {
        let p = Pagination::default();
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset, 0);
    }

    #[test]
    fn test_paginated_has_more() {
        let pagination = Pagination::new(10, 0);
        let page: Paginated<i32> = Paginated::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 25, &pagination);
        assert!(page.has_more());
        assert_eq!(page.next_offset(), 10);

        let pagination = Pagination::new(10, 20);
        let page: Paginated<i32> = Paginated::new(vec![1, 2, 3, 4, 5], 25, &pagination);
        assert!(!page.has_more());
    }
}
