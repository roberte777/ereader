//! API server binary entry point.

use api_server::{AppState, AppStateConfig, create_router};
use common::config::AppConfig;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting e-reader API server");

    // Load configuration
    let config = AppConfig::load()?;
    tracing::info!("loaded config: {:?}", config);

    tracing::info!(
        host = %config.server.host,
        port = %config.server.port,
        "Configuration loaded"
    );

    // Create database pool
    let pool = db_layer::create_pool(&config.database).await?;
    tracing::info!("Database connection pool created");

    // Run migrations
    db_layer::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // Create storage
    let storage = storage_layer::LocalStorage::from_config(&config.storage).await?;
    tracing::info!("Storage initialized");

    // Create app state
    let state_config = AppStateConfig {
        clerk_secret_key: config.clerk.secret_key.clone(),
        clerk_jwks_url: config.clerk.jwks_url.clone(),
    };
    let state = AppState::new(pool, storage, state_config);

    // Create router
    let app = create_router(state);

    // Create server address
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid server address");

    tracing::info!(%addr, "Starting HTTP server");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
