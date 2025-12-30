//! Device model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Device record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub device_type: String,
    pub public_key: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Data for creating a new device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDevice {
    pub id: Uuid,
    pub user_id: String,
    pub name: String,
    pub device_type: String,
    pub public_key: Option<String>,
}

impl CreateDevice {
    pub fn new(user_id: impl Into<String>, name: impl Into<String>, device_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::now_v7(),
            user_id: user_id.into(),
            name: name.into(),
            device_type: device_type.into(),
            public_key: None,
        }
    }

    pub fn with_public_key(mut self, key: impl Into<String>) -> Self {
        self.public_key = Some(key.into());
        self
    }
}
