//! Background task model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Unknown task status: {}", s)),
        }
    }
}

/// Task record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub task_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Task {
    /// Get the task status as an enum
    pub fn status(&self) -> TaskStatus {
        self.status.parse().unwrap_or(TaskStatus::Pending)
    }

    /// Check if the task can be retried
    pub fn can_retry(&self) -> bool {
        self.attempts < self.max_attempts
    }
}

/// Data for creating a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTask {
    pub task_type: String,
    pub payload: serde_json::Value,
    pub priority: i32,
    pub max_attempts: i32,
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl CreateTask {
    pub fn new(task_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            task_type: task_type.into(),
            payload,
            priority: 0,
            max_attempts: 3,
            scheduled_at: None,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_attempts(mut self, max_attempts: i32) -> Self {
        self.max_attempts = max_attempts;
        self
    }

    pub fn scheduled_at(mut self, time: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(time);
        self
    }
}

/// Known task types
pub mod task_types {
    pub const REINDEX_BOOK: &str = "reindex_book";
    pub const GENERATE_COVERS: &str = "generate_covers";
    pub const CLEANUP_ORPHANS: &str = "cleanup_orphans";
}
