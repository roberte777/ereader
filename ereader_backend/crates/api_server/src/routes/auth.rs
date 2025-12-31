//! Authentication and device management endpoints.

use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::state::AppState;

/// Request body for device registration
#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub name: String,
    pub device_type: String,
    #[serde(default)]
    pub public_key: Option<String>,
}

/// Response for device registration
#[derive(Debug, Serialize)]
pub struct DeviceResponse {
    pub device_id: Uuid,
    pub name: String,
    pub device_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Register a new device for the authenticated user
pub async fn register_device(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<DeviceResponse>, StatusCode> {
    let mut create_device = db_layer::models::CreateDevice::new(
        user.user_id.clone(),
        req.name.clone(),
        req.device_type.clone(),
    );

    if let Some(key) = req.public_key {
        create_device = create_device.with_public_key(key);
    }

    let device = db_layer::queries::DeviceQueries::create(&state.pool, &create_device)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to create device");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(DeviceResponse {
        device_id: device.id,
        name: device.name,
        device_type: device.device_type,
        created_at: device.created_at,
    }))
}

/// Webhook payload from Clerk
#[derive(Debug, Deserialize)]
pub struct ClerkWebhookEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: serde_json::Value,
}

/// Handle Clerk webhook events (user creation, updates, deletion)
pub async fn clerk_webhook(
    State(state): State<AppState>,
    Json(event): Json<ClerkWebhookEvent>,
) -> Result<StatusCode, StatusCode> {
    tracing::info!(event_type = %event.event_type, "Received Clerk webhook");

    match event.event_type.as_str() {
        "user.created" | "user.updated" => {
            // Extract user data and sync to database
            let user_id = event.data.get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    tracing::warn!("Missing user ID in webhook");
                    StatusCode::BAD_REQUEST
                })?;

            let email = event.data.get("email_addresses")
                .and_then(|arr| arr.as_array())
                .and_then(|arr| arr.first())
                .and_then(|obj| obj.get("email_address"))
                .and_then(|v| v.as_str());

            let name = event.data.get("first_name")
                .and_then(|v| v.as_str())
                .map(|first| {
                    let last = event.data.get("last_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    format!("{} {}", first, last).trim().to_string()
                });

            // Upsert user in database
            let create_user = db_layer::models::CreateUser {
                id: user_id.to_string(),
                email: email.map(|e| e.to_string()),
                name,
            };

            db_layer::queries::UserQueries::upsert(&state.pool, &create_user)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to sync user from webhook");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            Ok(StatusCode::OK)
        }
        "user.deleted" => {
            let user_id = event.data.get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    tracing::warn!("Missing user ID in webhook");
                    StatusCode::BAD_REQUEST
                })?;

            // Delete user from database (cascade will handle related data)
            db_layer::queries::UserQueries::delete(&state.pool, user_id)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "Failed to delete user from webhook");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            Ok(StatusCode::OK)
        }
        _ => {
            tracing::debug!(event_type = %event.event_type, "Ignoring webhook event");
            Ok(StatusCode::OK)
        }
    }
}
