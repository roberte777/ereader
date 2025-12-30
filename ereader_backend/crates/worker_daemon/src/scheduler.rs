//! Task scheduler for polling and executing background tasks.

use crate::tasks::{TaskHandler, TaskRegistry};
use db_layer::queries::TaskQueries;
use db_layer::DbPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use storage_layer::LocalStorage;

/// Configuration for the task scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub poll_interval: Duration,
    pub max_concurrent_tasks: usize,
    pub task_timeout: Duration,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            max_concurrent_tasks: 4,
            task_timeout: Duration::from_secs(300),
        }
    }
}

/// Context passed to task handlers
pub struct TaskContext {
    pub pool: DbPool,
    pub storage: Arc<LocalStorage>,
}

/// Task scheduler that polls for and executes background tasks
pub struct TaskScheduler {
    pool: DbPool,
    storage: Arc<LocalStorage>,
    config: SchedulerConfig,
    registry: TaskRegistry,
    semaphore: Arc<Semaphore>,
}

impl TaskScheduler {
    /// Create a new task scheduler
    pub fn new(
        pool: DbPool,
        storage: Arc<LocalStorage>,
        config: SchedulerConfig,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_tasks));
        let registry = TaskRegistry::new();

        Self {
            pool,
            storage,
            config,
            registry,
            semaphore,
        }
    }

    /// Register a task handler
    pub fn register_handler<H: TaskHandler + 'static>(&mut self, handler: H) {
        self.registry.register(handler);
    }

    /// Run the scheduler loop
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("Starting task scheduler");

        loop {
            // Poll for pending tasks
            match self.poll_and_execute().await {
                Ok(count) if count > 0 => {
                    tracing::debug!(tasks = count, "Processed tasks");
                }
                Ok(_) => {
                    // No tasks, wait before polling again
                    tokio::time::sleep(self.config.poll_interval).await;
                }
                Err(e) => {
                    tracing::error!(error = %e, "Error polling tasks");
                    tokio::time::sleep(self.config.poll_interval).await;
                }
            }
        }
    }

    /// Poll for and execute pending tasks
    async fn poll_and_execute(&self) -> anyhow::Result<usize> {
        let pending = TaskQueries::get_pending(&self.pool, 10).await?;

        if pending.is_empty() {
            return Ok(0);
        }

        let mut count = 0;

        for task in pending {
            // Acquire semaphore permit for concurrency control
            let permit = self.semaphore.clone().acquire_owned().await?;

            // Mark task as started
            if let Err(e) = TaskQueries::mark_started(&self.pool, task.id).await {
                tracing::error!(task_id = %task.id, error = %e, "Failed to mark task as started");
                continue;
            }

            // Get handler for this task type
            let handler = match self.registry.get(&task.task_type) {
                Some(h) => h,
                None => {
                    tracing::warn!(task_type = %task.task_type, "No handler for task type");
                    let _ = TaskQueries::mark_failed(
                        &self.pool,
                        task.id,
                        &format!("No handler for task type: {}", task.task_type),
                    ).await;
                    continue;
                }
            };

            // Clone values for the spawned task
            let pool = self.pool.clone();
            let storage = self.storage.clone();
            let task_id = task.id;
            let task_payload = task.payload.clone();
            let timeout = self.config.task_timeout;

            // Spawn task execution
            tokio::spawn(async move {
                let _permit = permit; // Hold permit until done

                let ctx = TaskContext {
                    pool: pool.clone(),
                    storage,
                };

                let result = tokio::time::timeout(timeout, handler.execute(&ctx, &task_payload)).await;

                match result {
                    Ok(Ok(_)) => {
                        if let Err(e) = TaskQueries::mark_completed(&pool, task_id).await {
                            tracing::error!(task_id = %task_id, error = %e, "Failed to mark task completed");
                        } else {
                            tracing::info!(task_id = %task_id, "Task completed successfully");
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!(task_id = %task_id, error = %e, "Task failed");
                        let _ = TaskQueries::mark_failed(&pool, task_id, &e.to_string()).await;
                    }
                    Err(_) => {
                        tracing::error!(task_id = %task_id, "Task timed out");
                        let _ = TaskQueries::mark_failed(&pool, task_id, "Task timed out").await;
                    }
                }
            });

            count += 1;
        }

        Ok(count)
    }
}
