# E-Reader Backend Implementation Plan

This document outlines a phased approach to implementing the e-reader API server based on the design document.

## Overview

The implementation is organized into 6 phases, each building on the previous. Each phase includes:
- Specific deliverables
- Testing requirements
- Verification steps

**Current State**: Empty workspace with only `Cargo.toml`, `CLAUDE.md`, and the design document.

---

## Phase 1: Foundation & Project Structure

### Objective
Set up the workspace structure, shared types, configuration, and error handling.

### Tasks

#### 1.1 Create Workspace Structure
- [ ] Create `crates/` directory
- [ ] Create all crate directories:
  - `crates/common`
  - `crates/db_layer`
  - `crates/storage_layer`
  - `crates/indexer`
  - `crates/sync_engine`
  - `crates/api_server`
  - `crates/api_client`
  - `crates/worker_daemon`
  - `crates/cli_tool`

#### 1.2 Configure Root Workspace
- [ ] Update root `Cargo.toml` with:
  - Workspace members list
  - `[workspace.package]` for shared metadata
  - `[workspace.dependencies]` for shared dependencies
- [ ] Create `.env.example` with all required environment variables
- [ ] Create `.gitignore` for Rust projects

#### 1.3 Implement `common` Crate
- [ ] Create `crates/common/Cargo.toml`
- [ ] Implement `src/lib.rs` - module exports
- [ ] Implement `src/types.rs`:
  - `BookId`, `UserId`, `DeviceId`, `FileAssetId`, `CollectionId`, `AnnotationId`
  - `ContentHash` with SHA-256 implementation
  - `ReadingLocation` struct
  - `BookFormat` enum with MIME types
  - `Pagination` and `Paginated<T>` types
- [ ] Implement `src/error.rs`:
  - Custom `Error` enum with all variants
  - `IntoResponse` implementation for Axum
  - `Result<T>` type alias
- [ ] Implement `src/config.rs`:
  - `AppConfig` with nested config structs
  - `ServerConfig`, `DatabaseConfig`, `StorageConfig`, `ClerkConfig`, `WorkerConfig`
  - Config loading from files and environment

#### 1.4 Create Configuration Files
- [ ] Create `config/` directory
- [ ] Create `config/default.toml` with all default settings
- [ ] Create `config/.gitkeep` for local config

### Verification
```bash
cargo check -p common
cargo test -p common
```

---

## Phase 2: Database Layer

### Objective
Implement database access, models, and queries using SQLx.

### Tasks

#### 2.1 Create Database Migrations
- [ ] Create `migrations/` directory at project root
- [ ] Create `migrations/20240101000000_initial_schema.sql`:
  - Enable uuid-ossp extension
  - Create custom types: `book_format`, `annotation_type`
  - Create `users` table
  - Create `devices` table
  - Create `books` table with indexes
  - Create `file_assets` table
  - Create `covers` table
  - Create `collections` table
  - Create `collection_books` junction table
  - Create `reading_states` table
  - Create `annotations` table
  - Create `tasks` table for background jobs
  - Create `update_updated_at_column()` trigger function
  - Apply triggers to relevant tables

#### 2.2 Implement `db_layer` Crate
- [ ] Create `crates/db_layer/Cargo.toml` with SQLx dependencies
- [ ] Implement `src/lib.rs` - module exports
- [ ] Implement `src/pool.rs`:
  - `DbPool` type alias
  - `create_pool()` function
  - `run_migrations()` function

#### 2.3 Implement Database Models
- [ ] Create `src/models/mod.rs`
- [ ] Implement `src/models/book.rs` - `Book`, `FileAsset`, `Cover`
- [ ] Implement `src/models/reading_state.rs` - `ReadingState`
- [ ] Implement `src/models/annotation.rs` - `AnnotationType`, `Annotation`
- [ ] Implement `src/models/device.rs` - `Device`
- [ ] Implement `src/models/collection.rs` - `Collection`

#### 2.4 Implement Database Queries
- [ ] Create `src/queries/mod.rs`
- [ ] Implement `src/queries/books.rs`:
  - `BookQueries::list_for_user()`
  - `BookQueries::get_by_id()`
  - `BookQueries::find_by_hash()`
  - `BookQueries::create()`
  - `BookQueries::update_metadata()`
  - `BookQueries::delete()`
  - `BookQueries::search()`
- [ ] Implement `src/queries/file_assets.rs`:
  - `create()`, `get_for_book()`, `delete()`
- [ ] Implement `src/queries/covers.rs`:
  - `create()`, `get_for_book()`, `delete_for_book()`
- [ ] Implement `src/queries/reading_states.rs`:
  - `get_for_book()`, `upsert()`, `get_updated_since()`
- [ ] Implement `src/queries/annotations.rs`:
  - `get_by_id()`, `list_for_user()`, `get_for_book()`
  - `upsert()`, `get_updated_since()`
- [ ] Implement `src/queries/devices.rs`:
  - `create()`, `get_by_id()`, `get_for_user()`
  - `update_last_sync()`
- [ ] Implement `src/queries/collections.rs`:
  - `list_for_user()`, `create()`, `update()`, `delete()`
  - `add_book()`, `remove_book()`, `get_books()`
- [ ] Implement `src/queries/tasks.rs`:
  - `create()`, `get_pending()`, `mark_started()`
  - `mark_completed()`, `mark_failed()`

### Verification
```bash
# Start PostgreSQL
docker compose up -d postgres

# Run migrations
sqlx migrate run

# Verify compile-time query checking
cargo sqlx prepare -- --lib -p db_layer
cargo check -p db_layer
cargo test -p db_layer
```

---

## Phase 3: Storage & Indexer

### Objective
Implement file storage abstraction and metadata extraction for ebooks.

### Tasks

#### 3.1 Implement `storage_layer` Crate
- [ ] Create `crates/storage_layer/Cargo.toml`
- [ ] Implement `src/lib.rs` - module exports
- [ ] Implement `src/traits.rs`:
  - `Storage` trait with async methods
  - `CoverStorage` trait for cover handling
  - `CoverPaths` struct
- [ ] Implement `src/local.rs`:
  - `LocalStorage` struct
  - `Storage` trait implementation
  - `CoverStorage` trait implementation with image resizing
  - Content-addressable file paths using hash prefixes

#### 3.2 Implement `indexer` Crate
- [ ] Create `crates/indexer/Cargo.toml` with epub/lopdf dependencies
- [ ] Implement `src/lib.rs` - module exports and `handler_for_format()`
- [ ] Implement `src/traits.rs`:
  - `BookMetadata` struct
  - `FormatHandler` trait
- [ ] Implement `src/epub.rs`:
  - `EpubHandler` struct
  - `extract_metadata()` - title, authors, description, etc.
  - `extract_cover()` - cover image extraction
  - `calculate_locations()` - spine items for navigation
- [ ] Implement `src/pdf.rs`:
  - `PdfHandler` struct
  - `extract_metadata()` - from PDF info dictionary
  - `extract_cover()` - placeholder (complex)
  - `calculate_locations()` - page numbers

### Verification
```bash
cargo check -p storage_layer -p indexer
cargo test -p storage_layer -p indexer
```

---

## Phase 4: Core API Server

### Objective
Implement the HTTP API server with authentication, routing, and core endpoints.

### Tasks

#### 4.1 Set Up `api_server` Crate Structure
- [ ] Create `crates/api_server/Cargo.toml`
- [ ] Implement `src/lib.rs`:
  - `AppState` struct with pool, storage, clerk_client
  - `create_app()` function with all routes
  - Layer configuration (CORS, compression, tracing, body limit)
- [ ] Implement `src/main.rs`:
  - Tracing initialization
  - Config loading
  - Database pool creation
  - Storage initialization
  - Server startup

#### 4.2 Implement Authentication Middleware
- [ ] Create `src/middleware/mod.rs`
- [ ] Implement `src/middleware/clerk.rs`:
  - `ClerkClient` struct with JWKS caching
  - `ClerkClaims` struct
  - `verify_token()` method
  - `AuthenticatedUser` struct
  - `auth_middleware()` function

#### 4.3 Implement Extractors
- [ ] Create `src/extractors/mod.rs`
- [ ] Implement `AuthUser` extractor that pulls from request extensions

#### 4.4 Implement Health Endpoints
- [ ] Create `src/routes/mod.rs`
- [ ] Implement `src/routes/health.rs`:
  - `health_check()` - liveness probe
  - `readiness_check()` - database connectivity

#### 4.5 Implement Library Endpoints
- [ ] Implement `src/routes/library.rs`:
  - `list_books()` with pagination and sorting
  - `get_book()` with details, formats, covers
  - `create_book()`
  - `update_book()`
  - `delete_book()` with cascade cleanup
  - `search_books()`
  - Request/Response DTOs

#### 4.6 Implement Asset Endpoints
- [ ] Implement `src/routes/assets.rs`:
  - `upload_file()` with multipart handling
  - `download_file()` - default format
  - `download_file_format()` - specific format
  - Content-type and disposition headers

#### 4.7 Implement Cover Endpoints
- [ ] Implement `src/routes/covers.rs`:
  - `get_cover()` - medium size default
  - `get_cover_size()` - specific size
  - `upload_cover()` - custom cover upload

#### 4.8 Implement Collection Endpoints
- [ ] Implement `src/routes/collections.rs`:
  - `list_collections()`
  - `create_collection()`
  - `get_collection()` with books
  - `update_collection()`
  - `delete_collection()`
  - `add_book()`
  - `remove_book()`

#### 4.9 Implement Error Handler
- [ ] Implement `src/error.rs`:
  - Convert common errors to API responses
  - Request ID tracking
  - Error logging

### Verification
```bash
cargo check -p api_server
cargo run --bin api_server
# Test endpoints with curl or httpie
```

---

## Phase 5: Sync Engine & Sync Endpoints

### Objective
Implement synchronization logic for reading states and annotations across devices.

### Tasks

#### 5.1 Implement `sync_engine` Crate
- [ ] Create `crates/sync_engine/Cargo.toml`
- [ ] Implement `src/lib.rs`:
  - `SyncRequest` struct
  - `ReadingStateSync` struct
  - `AnnotationSync` struct
  - `SyncResponse` struct
  - `SyncConflict` struct
  - `ConflictResolution` enum

#### 5.2 Implement Merge Logic
- [ ] Implement `src/merge.rs`:
  - `SyncMerger` struct
  - `process_sync()` main method
  - `merge_reading_states()` - LWW per book
  - `merge_annotations()` - LWW per annotation
  - `update_device_sync_time()`
  - Conflict detection and resolution

#### 5.3 Implement Batch Processing
- [ ] Implement `src/batch.rs`:
  - Batch update helpers
  - Transaction handling

#### 5.4 Implement Conflict Resolution
- [ ] Implement `src/conflicts.rs`:
  - Conflict detection logic
  - Resolution strategies (LWW, merge)

#### 5.5 Implement Sync Endpoints
- [ ] Implement `src/routes/sync.rs` in api_server:
  - `sync_batch()` - main sync endpoint
  - `get_reading_state()`
  - `update_reading_state()`
  - `list_annotations()`
  - `get_book_annotations()`

#### 5.6 Implement Auth/Device Endpoints
- [ ] Implement `src/routes/auth.rs`:
  - `register_device()` - device pairing
  - `clerk_webhook()` - user sync from Clerk

### Verification
```bash
cargo check -p sync_engine
cargo test -p sync_engine
# Integration tests with multiple simulated devices
```

---

## Phase 6: API Client & Infrastructure

### Objective
Implement the Rust client SDK and production infrastructure.

### Tasks

#### 6.1 Implement `api_client` Crate
- [ ] Create `crates/api_client/Cargo.toml`
- [ ] Implement `src/lib.rs` - module exports
- [ ] Implement `src/error.rs`:
  - Client-side `Error` enum
  - `is_retryable()`, `is_client_error()` methods
- [ ] Implement `src/client.rs`:
  - `ClientConfig` struct
  - `ClientBuilder` with fluent API
  - `Client` struct with endpoint accessors
  - Internal request method with retries, auth, caching

#### 6.2 Implement Auth Providers
- [ ] Implement `src/auth.rs`:
  - `AuthProvider` trait
  - `TokenAuth` - static token
  - `ClerkAuth` - with refresh
  - `DeviceAuth` - device tokens

#### 6.3 Implement Client Models
- [ ] Create `src/models/mod.rs`
- [ ] Implement `src/models/book.rs` - book types
- [ ] Implement `src/models/collection.rs` - collection types
- [ ] Implement `src/models/sync.rs` - sync types

#### 6.4 Implement Client Endpoints
- [ ] Create `src/endpoints/mod.rs`
- [ ] Implement `src/endpoints/books.rs`:
  - `BooksEndpoint` with builder pattern
  - `ListBooksBuilder`, `SearchBooksBuilder`
  - `CreateBookBuilder`, `UpdateBookBuilder`
- [ ] Implement `src/endpoints/assets.rs`:
  - `AssetsEndpoint` for uploads/downloads
- [ ] Implement `src/endpoints/collections.rs`:
  - `CollectionsEndpoint`
- [ ] Implement `src/endpoints/sync.rs`:
  - `SyncEndpoint` with `SyncBatchBuilder`
- [ ] Implement `src/endpoints/admin.rs`:
  - `AdminEndpoint`

#### 6.5 Implement Offline Support
- [ ] Create `src/offline/mod.rs`
- [ ] Implement `src/offline/cache.rs`:
  - `ResponseCache` with disk persistence
  - TTL handling
- [ ] Implement `src/offline/queue.rs`:
  - `OfflineQueue` for request queueing
  - Persistence to disk
  - Flush logic with retry

#### 6.6 Create Docker Infrastructure
- [ ] Create `docker-compose.yml`:
  - PostgreSQL service with health check
  - API service with environment config
  - Volumes for data persistence
- [ ] Create `Dockerfile`:
  - Multi-stage build (builder + runtime)
  - Dependency caching
  - Alpine-based runtime image

#### 6.7 Implement Admin Endpoints
- [ ] Implement `src/routes/admin.rs`:
  - `trigger_reindex()` - re-extract metadata
  - `create_backup()` - database backup
  - `get_stats()` - usage statistics

### Verification
```bash
cargo check -p api_client
cargo test -p api_client
docker compose up --build
```

---

## Phase 7: Worker, CLI & Polish

### Objective
Implement background processing, admin CLI, and finalize the project.

### Tasks

#### 7.1 Implement `worker_daemon` Crate
- [ ] Create `crates/worker_daemon/Cargo.toml`
- [ ] Implement `src/lib.rs`
- [ ] Implement `src/main.rs` - worker startup
- [ ] Implement `src/scheduler.rs`:
  - Task polling loop
  - Concurrency control
- [ ] Create `src/tasks/mod.rs`
- [ ] Implement task handlers:
  - `reindex_book` - re-extract metadata
  - `generate_covers` - regenerate cover sizes
  - `cleanup_orphans` - remove orphaned files

#### 7.2 Implement `cli_tool` Crate
- [ ] Create `crates/cli_tool/Cargo.toml` with clap
- [ ] Implement `src/main.rs` with clap derive
- [ ] Create `src/commands/mod.rs`
- [ ] Implement commands:
  - `user list` / `user create`
  - `book list` / `book import` / `book delete`
  - `migrate run` / `migrate status`
  - `backup create` / `backup restore`

#### 7.3 Comprehensive Testing
- [ ] Create `crates/api_server/tests/common/mod.rs` - test utilities
- [ ] Implement integration tests:
  - `library_test.rs` - CRUD operations
  - `sync_test.rs` - sync scenarios
  - `auth_test.rs` - authentication flows
- [ ] Add unit tests throughout crates

#### 7.4 Documentation & Polish
- [ ] Add rustdoc comments to public APIs
- [ ] Create README.md for root project
- [ ] Add example code snippets
- [ ] Review and update CLAUDE.md if needed

### Verification
```bash
cargo test --all
cargo clippy --all
cargo doc --all
```

---

## Testing Strategy Summary

### Unit Tests
- Each crate has inline `#[cfg(test)]` modules
- Focus on business logic in isolation

### Integration Tests
- Located in `crates/api_server/tests/`
- Use test database with migrations
- Mock Clerk authentication

### End-to-End Tests
- Full stack with Docker Compose
- Test real HTTP requests

---

## Development Workflow

### Daily Development
```bash
# Start database
docker compose up -d postgres

# Set environment
export DATABASE_URL=postgres://ereader:ereader_dev@localhost/ereader

# Run with hot reload
cargo watch -x 'run --bin api_server'

# Run tests
cargo test
```

### Before Commit
```bash
cargo fmt --all
cargo clippy --all
cargo test --all
cargo sqlx prepare -- --lib
```

---

## Dependencies Summary

### Core Dependencies
- **tokio**: Async runtime
- **axum**: Web framework
- **sqlx**: Database access with compile-time checking
- **serde**: Serialization
- **uuid**: UUID generation and parsing
- **chrono**: Date/time handling

### Authentication
- **jsonwebtoken**: JWT verification
- **reqwest**: HTTP client for JWKS

### Storage & Processing
- **sha2**: Content hashing
- **image**: Cover image resizing
- **epub**: EPUB parsing
- **lopdf**: PDF parsing

### CLI & Config
- **clap**: CLI argument parsing
- **config**: Configuration loading
- **tracing**: Structured logging

---

## Notes

- The workspace currently has `members = []` - this must be populated as crates are created
- SQLx requires the database to be running for compile-time query verification
- The design document specifies UUID v7 for time-ordering - ensure uuid crate is configured properly
- Clerk integration requires valid API keys for production testing
