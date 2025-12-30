//! Authentication providers for the API client.

use async_trait::async_trait;
use crate::error::Result;

/// Trait for authentication providers
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the authorization header value
    async fn get_auth_header(&self) -> Result<String>;

    /// Refresh the token if needed (returns true if refreshed)
    async fn refresh_if_needed(&self) -> Result<bool>;

    /// Check if authentication is available
    fn is_authenticated(&self) -> bool;
}

/// Static token authentication
pub struct TokenAuth {
    token: String,
}

impl TokenAuth {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

#[async_trait]
impl AuthProvider for TokenAuth {
    async fn get_auth_header(&self) -> Result<String> {
        Ok(format!("Bearer {}", self.token))
    }

    async fn refresh_if_needed(&self) -> Result<bool> {
        // Static tokens don't refresh
        Ok(false)
    }

    fn is_authenticated(&self) -> bool {
        !self.token.is_empty()
    }
}

/// Device token authentication (for offline-first clients)
pub struct DeviceAuth {
    device_id: String,
    device_token: String,
}

impl DeviceAuth {
    pub fn new(device_id: impl Into<String>, device_token: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            device_token: device_token.into(),
        }
    }

    pub fn device_id(&self) -> &str {
        &self.device_id
    }
}

#[async_trait]
impl AuthProvider for DeviceAuth {
    async fn get_auth_header(&self) -> Result<String> {
        Ok(format!("Bearer dev_{}", self.device_token))
    }

    async fn refresh_if_needed(&self) -> Result<bool> {
        // Device tokens don't refresh automatically
        Ok(false)
    }

    fn is_authenticated(&self) -> bool {
        !self.device_token.is_empty()
    }
}

/// No authentication (for public endpoints)
pub struct NoAuth;

#[async_trait]
impl AuthProvider for NoAuth {
    async fn get_auth_header(&self) -> Result<String> {
        Ok(String::new())
    }

    async fn refresh_if_needed(&self) -> Result<bool> {
        Ok(false)
    }

    fn is_authenticated(&self) -> bool {
        false
    }
}
