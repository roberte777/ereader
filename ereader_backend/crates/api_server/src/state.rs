//! Application state shared across handlers.

use crate::jwt::JwtValidator;
use db_layer::DbPool;
use std::sync::Arc;
use storage_layer::LocalStorage;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub storage: Arc<LocalStorage>,
    pub config: AppStateConfig,
    pub jwt_validator: JwtValidator,
}

/// Configuration subset needed by handlers
#[derive(Clone)]
pub struct AppStateConfig {
    pub clerk_secret_key: String,
    pub clerk_jwks_url: String,
}

impl AppState {
    pub fn new(pool: DbPool, storage: LocalStorage, config: AppStateConfig) -> Self {
        let jwt_validator = JwtValidator::new(
            Some(config.clerk_secret_key.clone()),
            Some(config.clerk_jwks_url.clone()),
        );

        Self {
            pool,
            storage: Arc::new(storage),
            config,
            jwt_validator,
        }
    }
}
