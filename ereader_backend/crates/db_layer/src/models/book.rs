//! Book, FileAsset, and Cover models.

use chrono::{DateTime, Utc};
use common::BookFormat;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Book record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Book {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
    // File fields (optional - book can exist without file)
    pub format: Option<BookFormat>,
    pub content_hash: Option<String>,
    pub file_size: Option<i64>,
    pub storage_path: Option<String>,
    pub original_filename: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Book {
    /// Check if this book has a file attached
    pub fn has_file(&self) -> bool {
        self.storage_path.is_some()
    }
}

/// Data for creating a new book
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateBook {
    pub id: Uuid,
    pub user_id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
    // File fields
    pub format: Option<BookFormat>,
    pub content_hash: Option<String>,
    pub file_size: Option<i64>,
    pub storage_path: Option<String>,
    pub original_filename: Option<String>,
}

impl CreateBook {
    pub fn new(user_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user_id.into(),
            title: title.into(),
            authors: vec![],
            description: None,
            language: None,
            publisher: None,
            published_date: None,
            isbn: None,
            series_name: None,
            series_index: None,
            tags: vec![],
            format: None,
            content_hash: None,
            file_size: None,
            storage_path: None,
            original_filename: None,
        }
    }

    pub fn with_authors(mut self, authors: Vec<String>) -> Self {
        self.authors = authors;
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.authors.push(author.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_series(mut self, name: impl Into<String>, index: f32) -> Self {
        self.series_name = Some(name.into());
        self.series_index = Some(index);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_file(
        mut self,
        format: BookFormat,
        content_hash: impl Into<String>,
        file_size: i64,
        storage_path: impl Into<String>,
        original_filename: impl Into<String>,
    ) -> Self {
        self.format = Some(format);
        self.content_hash = Some(content_hash.into());
        self.file_size = Some(file_size);
        self.storage_path = Some(storage_path.into());
        self.original_filename = Some(original_filename.into());
        self
    }
}

/// Data for updating an existing book
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateBook {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Option<Vec<String>>,
}

/// Cover record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cover {
    pub id: Uuid,
    pub book_id: Uuid,
    pub size_variant: String,
    pub width: i32,
    pub height: i32,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new cover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCover {
    pub book_id: Uuid,
    pub size_variant: String,
    pub width: i32,
    pub height: i32,
    pub storage_path: String,
}

impl CreateCover {
    pub fn new(
        book_id: Uuid,
        size_variant: impl Into<String>,
        width: i32,
        height: i32,
        storage_path: impl Into<String>,
    ) -> Self {
        Self {
            book_id,
            size_variant: size_variant.into(),
            width,
            height,
            storage_path: storage_path.into(),
        }
    }
}

/// Cover size variants
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
