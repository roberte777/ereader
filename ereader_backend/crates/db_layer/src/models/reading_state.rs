//! Reading state model.

use chrono::{DateTime, Utc};
use common::ReadingLocation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Reading state record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ReadingState {
    pub id: Uuid,
    pub user_id: String,
    pub book_id: Uuid,
    pub device_id: Uuid,
    pub location: sqlx::types::Json<ReadingLocation>,
    pub updated_at: DateTime<Utc>,
}

impl ReadingState {
    /// Get the reading location
    pub fn location(&self) -> &ReadingLocation {
        &self.location.0
    }
}

/// Data for creating or updating a reading state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertReadingState {
    pub user_id: String,
    pub book_id: Uuid,
    pub device_id: Uuid,
    pub location: ReadingLocation,
}

impl UpsertReadingState {
    pub fn new(
        user_id: impl Into<String>,
        book_id: Uuid,
        device_id: Uuid,
        location: ReadingLocation,
    ) -> Self {
        Self {
            user_id: user_id.into(),
            book_id,
            device_id,
            location,
        }
    }
}

/// Reading state with book information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStateWithBook {
    pub reading_state: ReadingState,
    pub book_title: String,
}
