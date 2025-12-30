//! Ebook indexer for metadata extraction and cover generation.
//!
//! This crate provides:
//! - Format-specific handlers for EPUB and PDF
//! - Metadata extraction (title, authors, description, etc.)
//! - Cover image extraction
//! - Location calculation for navigation

pub mod epub;
pub mod pdf;
pub mod traits;

pub use epub::EpubHandler;
pub use pdf::PdfHandler;
pub use traits::{BookMetadata, FormatHandler, LocationInfo, LocationItem};

use common::BookFormat;

/// Get the appropriate handler for a book format
pub fn handler_for_format(format: BookFormat) -> Option<Box<dyn FormatHandler>> {
    match format {
        BookFormat::Epub => Some(Box::new(EpubHandler::new())),
        BookFormat::Pdf => Some(Box::new(PdfHandler::new())),
        BookFormat::Cbz => None, // Not yet implemented
        BookFormat::Mobi => None, // Not yet implemented
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
        assert!(handler_for_format(BookFormat::Pdf).is_some());
        assert!(handler_for_format(BookFormat::Cbz).is_none());
        assert!(handler_for_format(BookFormat::Mobi).is_none());
    }
}
