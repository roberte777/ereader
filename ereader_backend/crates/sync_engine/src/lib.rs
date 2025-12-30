//! Sync engine for the e-reader API.
//!
//! This crate provides:
//! - Sync request/response types
//! - Last-write-wins (LWW) conflict resolution
//! - Batch sync processing

pub mod merge;
pub mod types;

pub use merge::SyncMerger;
pub use types::{
    AnnotationSync, AnnotationTypeSync, ConflictResolution, ReadingStateSync, SyncConflict,
    SyncRequest, SyncResponse,
};
