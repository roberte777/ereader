//! Sync request and response types.

use chrono::{DateTime, Utc};
use common::ReadingLocation;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sync request from a client device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub device_id: Uuid,
    #[serde(default)]
    pub last_sync_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reading_states: Vec<ReadingStateSync>,
    #[serde(default)]
    pub annotations: Vec<AnnotationSync>,
}

/// Reading state to sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStateSync {
    pub book_id: Uuid,
    pub location: ReadingLocation,
    pub updated_at: DateTime<Utc>,
}

/// Annotation type for sync
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationTypeSync {
    Highlight,
    Note,
    Bookmark,
}

impl From<common::AnnotationType> for AnnotationTypeSync {
    fn from(t: common::AnnotationType) -> Self {
        match t {
            common::AnnotationType::Highlight => Self::Highlight,
            common::AnnotationType::Note => Self::Note,
            common::AnnotationType::Bookmark => Self::Bookmark,
        }
    }
}

impl From<AnnotationTypeSync> for common::AnnotationType {
    fn from(t: AnnotationTypeSync) -> Self {
        match t {
            AnnotationTypeSync::Highlight => Self::Highlight,
            AnnotationTypeSync::Note => Self::Note,
            AnnotationTypeSync::Bookmark => Self::Bookmark,
        }
    }
}

/// Annotation to sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationSync {
    #[serde(default)]
    pub id: Option<Uuid>,
    pub book_id: Uuid,
    pub annotation_type: AnnotationTypeSync,
    pub location_start: String,
    #[serde(default)]
    pub location_end: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub style: Option<String>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub deleted: bool,
}

/// Sync response to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub server_time: DateTime<Utc>,
    pub reading_states: Vec<ReadingStateSync>,
    pub annotations: Vec<AnnotationSync>,
    pub conflicts: Vec<SyncConflict>,
}

impl SyncResponse {
    pub fn empty() -> Self {
        Self {
            server_time: Utc::now(),
            reading_states: vec![],
            annotations: vec![],
            conflicts: vec![],
        }
    }
}

/// Conflict detected during sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub local_updated_at: DateTime<Utc>,
    pub server_updated_at: DateTime<Utc>,
    pub resolution: ConflictResolution,
}

/// How a conflict was resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Merged,
}
