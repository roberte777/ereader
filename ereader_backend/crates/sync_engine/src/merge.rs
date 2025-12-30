//! Sync merge logic implementing last-write-wins (LWW) conflict resolution.

use crate::types::{
    AnnotationSync, ConflictResolution, ReadingStateSync, SyncConflict,
    SyncRequest, SyncResponse,
};
use chrono::{DateTime, Utc};
use common::Result;
use db_layer::{
    AnnotationQueries, DeviceQueries, DbPool, ReadingStateQueries,
    UpsertAnnotation, UpsertReadingState,
};
use uuid::Uuid;

/// Sync merger that processes sync requests
pub struct SyncMerger<'a> {
    pool: &'a DbPool,
}

impl<'a> SyncMerger<'a> {
    pub fn new(pool: &'a DbPool) -> Self {
        Self { pool }
    }

    /// Process a complete sync request
    pub async fn process_sync(
        &self,
        user_id: &str,
        request: SyncRequest,
    ) -> Result<SyncResponse> {
        let server_time = Utc::now();
        let last_sync_at = request.last_sync_at.unwrap_or_else(|| {
            DateTime::from_timestamp(0, 0).unwrap_or_else(Utc::now)
        });

        let mut conflicts = Vec::new();

        // Process reading states from client
        for state in &request.reading_states {
            if let Some(conflict) = self
                .merge_reading_state(user_id, request.device_id, state)
                .await?
            {
                conflicts.push(conflict);
            }
        }

        // Process annotations from client
        for annotation in &request.annotations {
            if let Some(conflict) = self
                .merge_annotation(user_id, annotation)
                .await?
            {
                conflicts.push(conflict);
            }
        }

        // Get updates from server since last sync
        let server_reading_states = self
            .get_reading_states_since(user_id, last_sync_at)
            .await?;
        let server_annotations = self
            .get_annotations_since(user_id, last_sync_at)
            .await?;

        // Update device last sync time
        DeviceQueries::update_last_sync(self.pool, request.device_id, server_time).await?;

        Ok(SyncResponse {
            server_time,
            reading_states: server_reading_states,
            annotations: server_annotations,
            conflicts,
        })
    }

    /// Merge a reading state using LWW
    async fn merge_reading_state(
        &self,
        user_id: &str,
        device_id: Uuid,
        client_state: &ReadingStateSync,
    ) -> Result<Option<SyncConflict>> {
        // Get current server state
        let server_state = ReadingStateQueries::get_for_book(
            self.pool,
            user_id,
            client_state.book_id,
        )
        .await?;

        let conflict = if let Some(ref server) = server_state {
            // Check for conflict (both modified)
            if server.updated_at > client_state.updated_at {
                // Server wins
                Some(SyncConflict {
                    entity_type: "reading_state".to_string(),
                    entity_id: client_state.book_id.to_string(),
                    local_updated_at: client_state.updated_at,
                    server_updated_at: server.updated_at,
                    resolution: ConflictResolution::ServerWins,
                })
            } else if server.updated_at < client_state.updated_at {
                // Client wins - update server
                let upsert = UpsertReadingState::new(
                    user_id,
                    client_state.book_id,
                    device_id,
                    client_state.location.clone(),
                );
                ReadingStateQueries::upsert(self.pool, &upsert).await?;
                None
            } else {
                // Same time - no conflict
                None
            }
        } else {
            // No server state - just insert
            let upsert = UpsertReadingState::new(
                user_id,
                client_state.book_id,
                device_id,
                client_state.location.clone(),
            );
            ReadingStateQueries::upsert(self.pool, &upsert).await?;
            None
        };

        Ok(conflict)
    }

    /// Merge an annotation using LWW
    async fn merge_annotation(
        &self,
        user_id: &str,
        client_annotation: &AnnotationSync,
    ) -> Result<Option<SyncConflict>> {
        let annotation_id = client_annotation.id.unwrap_or_else(Uuid::now_v7);

        // Get current server state
        let server_annotation = AnnotationQueries::get_by_id(self.pool, annotation_id).await?;

        let conflict = if let Some(ref server) = server_annotation {
            // Check for conflict
            if server.updated_at > client_annotation.updated_at {
                // Server wins
                Some(SyncConflict {
                    entity_type: "annotation".to_string(),
                    entity_id: annotation_id.to_string(),
                    local_updated_at: client_annotation.updated_at,
                    server_updated_at: server.updated_at,
                    resolution: ConflictResolution::ServerWins,
                })
            } else if server.updated_at < client_annotation.updated_at {
                // Client wins - update or delete
                if client_annotation.deleted {
                    AnnotationQueries::soft_delete(self.pool, annotation_id).await?;
                } else {
                    let upsert = UpsertAnnotation {
                        id: annotation_id,
                        user_id: user_id.to_string(),
                        book_id: client_annotation.book_id,
                        annotation_type: client_annotation.annotation_type.into(),
                        location_start: client_annotation.location_start.clone(),
                        location_end: client_annotation.location_end.clone(),
                        content: client_annotation.content.clone(),
                        style: client_annotation.style.clone(),
                    };
                    AnnotationQueries::upsert(self.pool, &upsert).await?;
                }
                None
            } else {
                // Same time - no conflict
                None
            }
        } else if !client_annotation.deleted {
            // No server state and not deleted - insert
            let upsert = UpsertAnnotation {
                id: annotation_id,
                user_id: user_id.to_string(),
                book_id: client_annotation.book_id,
                annotation_type: client_annotation.annotation_type.into(),
                location_start: client_annotation.location_start.clone(),
                location_end: client_annotation.location_end.clone(),
                content: client_annotation.content.clone(),
                style: client_annotation.style.clone(),
            };
            AnnotationQueries::upsert(self.pool, &upsert).await?;
            None
        } else {
            None
        };

        Ok(conflict)
    }

    /// Get reading states updated since a given time
    async fn get_reading_states_since(
        &self,
        user_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<ReadingStateSync>> {
        let states = ReadingStateQueries::get_updated_since(self.pool, user_id, since).await?;

        Ok(states
            .into_iter()
            .map(|s| ReadingStateSync {
                book_id: s.book_id,
                location: s.location.0.clone(),
                updated_at: s.updated_at,
            })
            .collect())
    }

    /// Get annotations updated since a given time
    async fn get_annotations_since(
        &self,
        user_id: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<AnnotationSync>> {
        let annotations = AnnotationQueries::get_updated_since(self.pool, user_id, since).await?;

        Ok(annotations
            .into_iter()
            .map(|a| AnnotationSync {
                id: Some(a.id),
                book_id: a.book_id,
                annotation_type: a.annotation_type.into(),
                location_start: a.location_start,
                location_end: a.location_end,
                content: a.content,
                style: a.style,
                updated_at: a.updated_at,
                deleted: a.deleted_at.is_some(),
            })
            .collect())
    }
}
