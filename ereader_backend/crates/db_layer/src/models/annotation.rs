//! Annotation model.

use chrono::{DateTime, Utc};
use common::AnnotationType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Annotation record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Annotation {
    pub id: Uuid,
    pub user_id: String,
    pub book_id: Uuid,
    pub annotation_type: AnnotationType,
    pub location_start: String,
    pub location_end: Option<String>,
    pub content: Option<String>,
    pub style: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Annotation {
    /// Check if this annotation has been soft-deleted
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// Data for creating or updating an annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertAnnotation {
    pub id: Uuid,
    pub user_id: String,
    pub book_id: Uuid,
    pub annotation_type: AnnotationType,
    pub location_start: String,
    pub location_end: Option<String>,
    pub content: Option<String>,
    pub style: Option<String>,
}

impl UpsertAnnotation {
    pub fn highlight(
        user_id: impl Into<String>,
        book_id: Uuid,
        location_start: impl Into<String>,
        location_end: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user_id.into(),
            book_id,
            annotation_type: AnnotationType::Highlight,
            location_start: location_start.into(),
            location_end: Some(location_end.into()),
            content: None,
            style: Some("yellow".to_string()),
        }
    }

    pub fn note(
        user_id: impl Into<String>,
        book_id: Uuid,
        location_start: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user_id.into(),
            book_id,
            annotation_type: AnnotationType::Note,
            location_start: location_start.into(),
            location_end: None,
            content: Some(content.into()),
            style: None,
        }
    }

    pub fn bookmark(user_id: impl Into<String>, book_id: Uuid, location: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user_id.into(),
            book_id,
            annotation_type: AnnotationType::Bookmark,
            location_start: location.into(),
            location_end: None,
            content: None,
            style: None,
        }
    }

    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}
