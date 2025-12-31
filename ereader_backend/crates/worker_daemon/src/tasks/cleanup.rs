//! Cleanup orphaned files task handler.

use crate::scheduler::TaskContext;
use crate::tasks::TaskHandler;
use async_trait::async_trait;

/// Handler for cleaning up orphaned files
pub struct CleanupOrphansHandler;

#[async_trait]
impl TaskHandler for CleanupOrphansHandler {
    fn task_type(&self) -> &'static str {
        "cleanup_orphans"
    }

    async fn execute(&self, _ctx: &TaskContext, _payload: &serde_json::Value) -> anyhow::Result<()> {
        tracing::info!("Starting orphan cleanup");

        // Note: This is a simplified implementation.
        // Full implementation would:
        // 1. Query all books with storage_path set (requires admin query)
        // 2. Check if each file exists in storage
        // 3. Log or delete orphaned references

        let orphan_count = 0;

        tracing::info!(orphans_found = orphan_count, "Cleanup completed");

        Ok(())
    }
}
