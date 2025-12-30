//! PDF format handler.

use crate::traits::{BookMetadata, FormatHandler, LocationInfo, LocationItem};
use common::{BookFormat, Error, Result};
use lopdf::Document;
use std::io::Cursor;

/// Handler for PDF files
pub struct PdfHandler;

impl PdfHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PdfHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatHandler for PdfHandler {
    fn format(&self) -> BookFormat {
        BookFormat::Pdf
    }

    fn extract_metadata(&self, data: &[u8]) -> Result<BookMetadata> {
        let cursor = Cursor::new(data);
        let doc = Document::load_from(cursor)
            .map_err(|e| Error::Validation(format!("Failed to parse PDF: {}", e)))?;

        let mut metadata = BookMetadata::default();

        // Try to get info dictionary
        if let Ok(info) = doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info.as_reference() {
                if let Ok(info_dict) = doc.get_dictionary(info_ref) {
                    // Title
                    if let Ok(title) = info_dict.get(b"Title") {
                        if let Ok(title_bytes) = title.as_str() {
                            if let Ok(title_str) = std::str::from_utf8(title_bytes) {
                                metadata.title = Some(title_str.to_string());
                            } else {
                                // Try lossy conversion for non-UTF8
                                metadata.title = Some(String::from_utf8_lossy(title_bytes).to_string());
                            }
                        }
                    }

                    // Author
                    if let Ok(author) = info_dict.get(b"Author") {
                        if let Ok(author_bytes) = author.as_str() {
                            let author_string = String::from_utf8_lossy(author_bytes).to_string();
                            metadata.authors = author_string
                                .split(&[',', ';', '&'][..])
                                .map(|s: &str| s.trim().to_string())
                                .filter(|s: &String| !s.is_empty())
                                .collect();
                        }
                    }

                    // Subject (description)
                    if let Ok(subject) = info_dict.get(b"Subject") {
                        if let Ok(subject_bytes) = subject.as_str() {
                            metadata.description = Some(String::from_utf8_lossy(subject_bytes).to_string());
                        }
                    }

                    // Keywords (subjects/tags)
                    if let Ok(keywords) = info_dict.get(b"Keywords") {
                        if let Ok(keywords_bytes) = keywords.as_str() {
                            let keywords_string = String::from_utf8_lossy(keywords_bytes).to_string();
                            metadata.subjects = keywords_string
                                .split(&[',', ';'][..])
                                .map(|s: &str| s.trim().to_string())
                                .filter(|s: &String| !s.is_empty())
                                .collect();
                        }
                    }
                }
            }
        }

        tracing::debug!(
            title = ?metadata.title,
            authors = ?metadata.authors,
            "Extracted PDF metadata"
        );

        Ok(metadata)
    }

    fn extract_cover(&self, _data: &[u8]) -> Result<Option<Vec<u8>>> {
        // PDF cover extraction is complex - would need to render first page
        // For now, return None and let the frontend handle it
        tracing::debug!("PDF cover extraction not implemented");
        Ok(None)
    }

    fn calculate_locations(&self, data: &[u8]) -> Result<LocationInfo> {
        let cursor = Cursor::new(data);
        let doc = Document::load_from(cursor)
            .map_err(|e| Error::Validation(format!("Failed to parse PDF: {}", e)))?;

        let page_count = doc.get_pages().len();

        let items: Vec<LocationItem> = (1..=page_count)
            .map(|page_num| LocationItem {
                id: format!("page:{}", page_num),
                label: Some(format!("Page {}", page_num)),
                order: page_num as u32,
            })
            .collect();

        tracing::debug!(
            pages = page_count,
            "Calculated PDF locations"
        );

        Ok(LocationInfo {
            total_locations: page_count as u32,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_handler_format() {
        let handler = PdfHandler::new();
        assert_eq!(handler.format(), BookFormat::Pdf);
    }
}
