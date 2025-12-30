//! EPUB format handler.

use crate::traits::{BookMetadata, FormatHandler, LocationInfo, LocationItem};
use common::{BookFormat, Error, Result};
use epub::doc::EpubDoc;
use std::io::Cursor;

/// Handler for EPUB files
pub struct EpubHandler;

impl EpubHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EpubHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatHandler for EpubHandler {
    fn format(&self) -> BookFormat {
        BookFormat::Epub
    }

    fn extract_metadata(&self, data: &[u8]) -> Result<BookMetadata> {
        let cursor = Cursor::new(data);
        let doc = EpubDoc::from_reader(cursor)
            .map_err(|e| Error::Validation(format!("Failed to parse EPUB: {}", e)))?;

        let mut metadata = BookMetadata::default();

        // Title
        if let Some(title) = doc.mdata("title") {
            metadata.title = Some(title.value.clone());
        }

        // Authors (creator)
        if let Some(creator) = doc.mdata("creator") {
            // Split multiple authors if separated by common delimiters
            metadata.authors = creator.value
                .split(&[',', ';', '&'][..])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Description
        if let Some(desc) = doc.mdata("description") {
            metadata.description = Some(desc.value.clone());
        }

        // Language
        if let Some(lang) = doc.mdata("language") {
            metadata.language = Some(lang.value.clone());
        }

        // Publisher
        if let Some(pub_) = doc.mdata("publisher") {
            metadata.publisher = Some(pub_.value.clone());
        }

        // Date
        if let Some(date) = doc.mdata("date") {
            metadata.published_date = Some(date.value.clone());
        }

        // ISBN (check various identifier types)
        if let Some(isbn) = doc.mdata("identifier") {
            // Only use if it looks like an ISBN
            if isbn.value.chars().all(|c| c.is_ascii_digit() || c == '-' || c == 'X' || c == 'x') {
                metadata.isbn = Some(isbn.value.clone());
            }
        }

        // Subjects/Tags
        if let Some(subject) = doc.mdata("subject") {
            metadata.subjects = subject.value
                .split(&[',', ';'][..])
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        tracing::debug!(
            title = ?metadata.title,
            authors = ?metadata.authors,
            "Extracted EPUB metadata"
        );

        Ok(metadata)
    }

    fn extract_cover(&self, data: &[u8]) -> Result<Option<Vec<u8>>> {
        let cursor = Cursor::new(data);
        let mut doc = EpubDoc::from_reader(cursor)
            .map_err(|e| Error::Validation(format!("Failed to parse EPUB: {}", e)))?;

        // Try to get the cover image
        match doc.get_cover() {
            Some((cover_data, _mime_type)) => {
                tracing::debug!(size = cover_data.len(), "Extracted EPUB cover");
                Ok(Some(cover_data))
            }
            None => {
                tracing::debug!("No cover found in EPUB");
                Ok(None)
            }
        }
    }

    fn calculate_locations(&self, data: &[u8]) -> Result<LocationInfo> {
        let cursor = Cursor::new(data);
        let doc = EpubDoc::from_reader(cursor)
            .map_err(|e| Error::Validation(format!("Failed to parse EPUB: {}", e)))?;

        let spine = doc.spine.clone();
        let mut items = Vec::new();

        for (order, spine_item) in spine.iter().enumerate() {
            items.push(LocationItem {
                id: spine_item.idref.clone(),
                label: None,  // Could extract from NCX/NAV
                order: order as u32,
            });
        }

        let total_locations = items.len() as u32;

        tracing::debug!(
            spine_items = items.len(),
            "Calculated EPUB locations"
        );

        Ok(LocationInfo {
            total_locations,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epub_handler_format() {
        let handler = EpubHandler::new();
        assert_eq!(handler.format(), BookFormat::Epub);
    }
}
