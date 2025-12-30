//! Cleanup orphaned files task handler.

use crate::scheduler::TaskContext;
use crate::tasks::TaskHandler;
use async_trait::async_trait;
use storage_layer::traits::Storage;

/// Handler for cleaning up orphaned files
pub struct CleanupOrphansHandler;

#[async_trait]
impl TaskHandler for CleanupOrphansHandler {
    fn task_type(&self) -> &'static str {
        "cleanup_orphans"
    }

    async fn execute(&self, ctx: &TaskContext, _payload: &serde_json::Value) -> anyhow::Result<()> {
        tracing::info!("Starting orphan cleanup");

        // Get all file assets from database
        let file_assets = db_layer::queries::FileAssetQueries::get_all(&ctx.pool).await?;

        let mut deleted_count = 0;

        // Check each file asset
        for asset in file_assets {
            // Check if file exists in storage
            let exists = ctx.storage.exists(&asset.storage_path).await?;

            if !exists {
                tracing::warn!(
                    asset_id = %asset.id,
                    storage_path = %asset.storage_path,
                    "File asset references missing file"
                );
                // Optionally delete the database record
                // db_layer::queries::FileAssetQueries::delete(&ctx.pool, asset.id).await?;
                deleted_count += 1;
            }
        }

        tracing::info!(orphans_found = deleted_count, "Cleanup completed");

        Ok(())
    }
}
