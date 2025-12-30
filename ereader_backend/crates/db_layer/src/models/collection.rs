//! Collection model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Collection record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Collection {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub collection_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCollection {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub collection_type: String,
}

impl CreateCollection {
    pub fn new(user_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user_id.into(),
            name: name.into(),
            description: None,
            collection_type: "shelf".to_string(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_type(mut self, collection_type: impl Into<String>) -> Self {
        self.collection_type = collection_type.into();
        self
    }
}

/// Data for updating an existing collection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateCollection {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Collection book membership record
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CollectionBook {
    pub collection_id: Uuid,
    pub book_id: Uuid,
    pub added_at: DateTime<Utc>,
    pub sort_order: Option<i32>,
}

/// Collection types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionType {
    Shelf,
    Tag,
    Series,
}

impl CollectionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shelf => "shelf",
            Self::Tag => "tag",
            Self::Series => "series",
        }
    }
}

impl std::fmt::Display for CollectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
