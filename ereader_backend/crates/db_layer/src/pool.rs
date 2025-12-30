//! Database connection pool and migration utilities.

use common::config::DatabaseConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Type alias for the PostgreSQL connection pool
pub type DbPool = PgPool;

/// Create a new database connection pool from configuration
pub async fn create_pool(config: &DatabaseConfig) -> Result<DbPool, sqlx::Error> {
    tracing::info!(
        max_connections = config.max_connections,
        min_connections = config.min_connections,
        "Creating database connection pool"
    );

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.url)
        .await?;

    tracing::info!("Database connection pool created successfully");
    Ok(pool)
}

/// Create a connection pool from a database URL string
pub async fn create_pool_from_url(url: &str) -> Result<DbPool, sqlx::Error> {
    let config = DatabaseConfig {
        url: url.to_string(),
        max_connections: 10,
        min_connections: 2,
    };
    create_pool(&config).await
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running database migrations");
    sqlx::migrate!("../../migrations").run(pool).await?;
    tracing::info!("Database migrations completed successfully");
    Ok(())
}

/// Check if the database is ready to accept connections
pub async fn health_check(pool: &DbPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }
}
