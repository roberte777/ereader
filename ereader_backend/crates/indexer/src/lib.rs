//! Ebook indexer for metadata extraction and cover generation.
//!
//! This crate provides:
//! - Format-specific handlers (currently EPUB only)
//! - Metadata extraction (title, authors, description, etc.)
//! - Cover image extraction
//! - Location calculation for navigation
//!
//! To add support for new formats in the future:
//! 1. Create a new handler module (e.g., `pdf.rs`)
//! 2. Implement the `FormatHandler` trait
//! 3. Add the handler to `handler_for_format()`

pub mod epub;
pub mod traits;

pub use epub::EpubHandler;
pub use traits::{BookMetadata, FormatHandler, LocationInfo, LocationItem};

use common::BookFormat;

/// Get the appropriate handler for a book format
pub fn handler_for_format(format: BookFormat) -> Option<Box<dyn FormatHandler>> {
    match format {
        BookFormat::Epub => Some(Box::new(EpubHandler::new())),
    }
}

/// Extract metadata from ebook data based on format
pub fn extract_metadata(format: BookFormat, data: &[u8]) -> common::Result<BookMetadata> {
    let handler = handler_for_format(format)
        .ok_or_else(|| common::Error::Validation(format!("Unsupported format: {}", format)))?;
    handler.extract_metadata(data)
}

/// Extract cover image from ebook data based on format
pub fn extract_cover(format: BookFormat, data: &[u8]) -> common::Result<Option<Vec<u8>>> {
    let handler = handler_for_format(format)
        .ok_or_else(|| common::Error::Validation(format!("Unsupported format: {}", format)))?;
    handler.extract_cover(data)
}

/// Calculate locations from ebook data based on format
pub fn calculate_locations(format: BookFormat, data: &[u8]) -> common::Result<LocationInfo> {
    let handler = handler_for_format(format)
        .ok_or_else(|| common::Error::Validation(format!("Unsupported format: {}", format)))?;
    handler.calculate_locations(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_for_format() {
        assert!(handler_for_format(BookFormat::Epub).is_some());
    }
}
