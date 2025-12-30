//! Task database queries.

use crate::models::{CreateTask, Task, TaskStatus};
use crate::pool::DbPool;
use chrono::{DateTime, Utc};
use common::Result;
use uuid::Uuid;

/// Task-related database queries
pub struct TaskQueries;

impl TaskQueries {
    /// Get a task by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Task>> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, payload, status, priority, attempts, max_attempts,
                   scheduled_at, started_at, completed_at, error, created_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(task)
    }

    /// Create a new task
    pub async fn create(pool: &DbPool, data: &CreateTask) -> Result<Task> {
        let task = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (task_type, payload, priority, max_attempts, scheduled_at)
            VALUES ($1, $2, $3, $4, COALESCE($5, NOW()))
            RETURNING id, task_type, payload, status, priority, attempts, max_attempts,
                      scheduled_at, started_at, completed_at, error, created_at
            "#,
        )
        .bind(&data.task_type)
        .bind(&data.payload)
        .bind(data.priority)
        .bind(data.max_attempts)
        .bind(data.scheduled_at)
        .fetch_one(pool)
        .await?;

        Ok(task)
    }

    /// Get pending tasks ready to be executed
    pub async fn get_pending(pool: &DbPool, limit: i64) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, payload, status, priority, attempts, max_attempts,
                   scheduled_at, started_at, completed_at, error, created_at
            FROM tasks
            WHERE status = 'pending' AND scheduled_at <= NOW()
            ORDER BY priority DESC, scheduled_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(tasks)
    }

    /// Mark a task as started
    pub async fn mark_started(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET status = 'running', started_at = NOW(), attempts = attempts + 1
            WHERE id = $1 AND status = 'pending'
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark a task as completed
    pub async fn mark_completed(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET status = 'completed', completed_at = NOW()
            WHERE id = $1 AND status = 'running'
            "#,
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Mark a task as failed
    pub async fn mark_failed(pool: &DbPool, id: Uuid, error: &str) -> Result<bool> {
        // First check if the task can be retried
        let task = Self::get_by_id(pool, id).await?;

        let new_status = if let Some(task) = task {
            if task.attempts < task.max_attempts {
                TaskStatus::Pending // Can retry
            } else {
                TaskStatus::Failed // Max attempts reached
            }
        } else {
            return Ok(false);
        };

        let result = sqlx::query(
            r#"
            UPDATE tasks
            SET status = $2, error = $3, completed_at = CASE WHEN $2 = 'failed' THEN NOW() ELSE NULL END
            WHERE id = $1 AND status = 'running'
            "#,
        )
        .bind(id)
        .bind(new_status.as_str())
        .bind(error)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get tasks by status
    pub async fn get_by_status(pool: &DbPool, status: TaskStatus) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, payload, status, priority, attempts, max_attempts,
                   scheduled_at, started_at, completed_at, error, created_at
            FROM tasks
            WHERE status = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(status.as_str())
        .fetch_all(pool)
        .await?;

        Ok(tasks)
    }

    /// Get tasks by type
    pub async fn get_by_type(pool: &DbPool, task_type: &str) -> Result<Vec<Task>> {
        let tasks = sqlx::query_as::<_, Task>(
            r#"
            SELECT id, task_type, payload, status, priority, attempts, max_attempts,
                   scheduled_at, started_at, completed_at, error, created_at
            FROM tasks
            WHERE task_type = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(task_type)
        .fetch_all(pool)
        .await?;

        Ok(tasks)
    }

    /// Delete old completed tasks
    pub async fn cleanup_completed(pool: &DbPool, older_than: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM tasks WHERE status = 'completed' AND completed_at < $1",
        )
        .bind(older_than)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete old failed tasks
    pub async fn cleanup_failed(pool: &DbPool, older_than: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM tasks WHERE status = 'failed' AND completed_at < $1",
        )
        .bind(older_than)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}
