//! Background task handlers.

pub mod reindex;
pub mod covers;
pub mod cleanup;

use crate::scheduler::TaskContext;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub use reindex::ReindexBookHandler;
pub use covers::GenerateCoversHandler;
pub use cleanup::CleanupOrphansHandler;

/// Trait for task handlers
#[async_trait]
pub trait TaskHandler: Send + Sync {
    /// Get the task type this handler handles
    fn task_type(&self) -> &'static str;

    /// Execute the task with the given payload
    async fn execute(&self, ctx: &TaskContext, payload: &serde_json::Value) -> anyhow::Result<()>;
}

/// Registry of task handlers
pub struct TaskRegistry {
    handlers: HashMap<String, Arc<dyn TaskHandler>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler
    pub fn register<H: TaskHandler + 'static>(&mut self, handler: H) {
        let task_type = handler.task_type().to_string();
        self.handlers.insert(task_type, Arc::new(handler));
    }

    /// Get a handler by task type
    pub fn get(&self, task_type: &str) -> Option<Arc<dyn TaskHandler>> {
        self.handlers.get(task_type).cloned()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}
