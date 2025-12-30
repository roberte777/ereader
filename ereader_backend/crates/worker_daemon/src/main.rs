//! Worker daemon binary entry point.

use common::config::AppConfig;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker_daemon::{
    TaskScheduler,
    scheduler::SchedulerConfig,
    tasks::{CleanupOrphansHandler, GenerateCoversHandler, ReindexBookHandler},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting worker daemon");

    // Load configuration
    let config = AppConfig::load()?;

    tracing::info!("Configuration loaded");

    // Create database pool
    let pool = db_layer::create_pool(&config.database).await?;
    tracing::info!("Database connection pool created");

    // Create storage
    let storage = storage_layer::LocalStorage::from_config(&config.storage).await?;
    let storage = Arc::new(storage);
    tracing::info!("Storage initialized");

    // Create scheduler config
    let scheduler_config = SchedulerConfig {
        poll_interval: std::time::Duration::from_secs(config.worker.poll_interval_secs.into()),
        max_concurrent_tasks: config.worker.max_concurrent_tasks as usize,
        task_timeout: std::time::Duration::from_secs(config.worker.task_timeout_secs.into()),
    };

    // Create scheduler
    let mut scheduler = TaskScheduler::new(pool, storage, scheduler_config);

    // Register task handlers
    scheduler.register_handler(ReindexBookHandler);
    scheduler.register_handler(GenerateCoversHandler);
    scheduler.register_handler(CleanupOrphansHandler);

    tracing::info!("Task handlers registered");

    // Run the scheduler
    scheduler.run().await?;

    Ok(())
}
