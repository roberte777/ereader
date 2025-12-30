//! Indexer trait definitions.

use common::{BookFormat, Result};

/// Extracted metadata from an ebook
#[derive(Debug, Clone, Default)]
pub struct BookMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub subjects: Vec<String>,
}

impl BookMetadata {
    /// Get the title or a default
    pub fn title_or_default(&self, default: &str) -> String {
        self.title.clone().unwrap_or_else(|| default.to_string())
    }

    /// Check if we have any meaningful metadata
    pub fn has_data(&self) -> bool {
        self.title.is_some()
            || !self.authors.is_empty()
            || self.description.is_some()
    }
}

/// Location information for navigation
#[derive(Debug, Clone)]
pub struct LocationInfo {
    /// Number of locations/pages
    pub total_locations: u32,
    /// Spine/chapter items for EPUB, page numbers for PDF
    pub items: Vec<LocationItem>,
}

#[derive(Debug, Clone)]
pub struct LocationItem {
    /// Item identifier (spine id for EPUB, page number for PDF)
    pub id: String,
    /// Human-readable label
    pub label: Option<String>,
    /// Order in the book
    pub order: u32,
}

/// Trait for format-specific ebook handlers
pub trait FormatHandler: Send + Sync {
    /// Get the format this handler supports
    fn format(&self) -> BookFormat;

    /// Extract metadata from the ebook data
    fn extract_metadata(&self, data: &[u8]) -> Result<BookMetadata>;

    /// Extract cover image from the ebook
    fn extract_cover(&self, data: &[u8]) -> Result<Option<Vec<u8>>>;

    /// Calculate location information for navigation
    fn calculate_locations(&self, data: &[u8]) -> Result<LocationInfo>;
}
