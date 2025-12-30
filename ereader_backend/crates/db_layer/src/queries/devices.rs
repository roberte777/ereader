//! Device database queries.

use crate::models::{CreateDevice, Device};
use crate::pool::DbPool;
use chrono::{DateTime, Utc};
use common::{Error, Result};
use uuid::Uuid;

/// Device-related database queries
pub struct DeviceQueries;

impl DeviceQueries {
    /// Get a device by ID
    pub async fn get_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Device>> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            SELECT id, user_id, name, device_type, public_key, last_sync_at, created_at
            FROM devices
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(device)
    }

    /// Get a device by ID, returning an error if not found
    pub async fn get_by_id_required(pool: &DbPool, id: Uuid) -> Result<Device> {
        Self::get_by_id(pool, id)
            .await?
            .ok_or_else(|| Error::not_found_resource("device", id))
    }

    /// Get all devices for a user
    pub async fn get_for_user(pool: &DbPool, user_id: &str) -> Result<Vec<Device>> {
        let devices = sqlx::query_as::<_, Device>(
            r#"
            SELECT id, user_id, name, device_type, public_key, last_sync_at, created_at
            FROM devices
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(devices)
    }

    /// Create a new device
    pub async fn create(pool: &DbPool, data: &CreateDevice) -> Result<Device> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            INSERT INTO devices (id, user_id, name, device_type, public_key)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, device_type, public_key, last_sync_at, created_at
            "#,
        )
        .bind(data.id)
        .bind(&data.user_id)
        .bind(&data.name)
        .bind(&data.device_type)
        .bind(&data.public_key)
        .fetch_one(pool)
        .await?;

        Ok(device)
    }

    /// Update the last sync time for a device
    pub async fn update_last_sync(pool: &DbPool, id: Uuid, time: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE devices
            SET last_sync_at = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(time)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Delete a device
    pub async fn delete(pool: &DbPool, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM devices WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
