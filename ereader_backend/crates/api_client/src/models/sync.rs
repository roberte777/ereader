//! Sync-related API models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sync request to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub device_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub reading_states: Vec<ReadingStateSync>,
    #[serde(default)]
    pub annotations: Vec<AnnotationSync>,
}

impl SyncRequest {
    pub fn new(device_id: Uuid) -> Self {
        Self {
            device_id,
            last_sync_at: None,
            reading_states: vec![],
            annotations: vec![],
        }
    }

    pub fn last_sync_at(mut self, time: DateTime<Utc>) -> Self {
        self.last_sync_at = Some(time);
        self
    }

    pub fn with_reading_states(mut self, states: Vec<ReadingStateSync>) -> Self {
        self.reading_states = states;
        self
    }

    pub fn with_annotations(mut self, annotations: Vec<AnnotationSync>) -> Self {
        self.annotations = annotations;
        self
    }
}

/// Reading state for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStateSync {
    pub book_id: Uuid,
    pub current_location: String,
    pub progress_percent: f32,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Annotation for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationSync {
    pub id: Uuid,
    pub book_id: Uuid,
    pub annotation_type: AnnotationType,
    pub location: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i64,
}

/// Annotation type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
}

/// Sync response from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub server_time: DateTime<Utc>,
    pub reading_states: Vec<ReadingStateSync>,
    pub annotations: Vec<AnnotationSync>,
    pub conflicts: Vec<SyncConflict>,
}

/// Sync conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub resolution: ConflictResolution,
    pub server_version: i64,
    pub client_version: i64,
}

/// How the conflict was resolved
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Merged,
}

/// Device registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceRequest {
    pub name: String,
    pub device_type: String,
    pub platform: String,
}

impl RegisterDeviceRequest {
    pub fn new(name: impl Into<String>, device_type: impl Into<String>, platform: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            device_type: device_type.into(),
            platform: platform.into(),
        }
    }
}

/// Device registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterDeviceResponse {
    pub device_id: Uuid,
    pub device_token: String,
}
