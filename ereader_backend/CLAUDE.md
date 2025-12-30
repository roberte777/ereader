# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an e-reader API server backend built with Rust. The system manages ebook files, metadata, user libraries, collections, reading progress, and synchronizes state across multiple devices.

**Technology Stack:**
- Language: Rust
- Web Framework: Axum
- Database: PostgreSQL with SQLx (compile-time checked queries)
- Authentication: Clerk (managed auth with device tokens)
- Task Queue: tokio-based internal queue
- File Storage: Local filesystem (abstracted for future S3 support)

## Workspace Architecture

This is a Cargo workspace with multiple crates under `crates/`:

- **common**: Shared types, error handling, and configuration
- **api_client**: Rust client SDK for the API with offline support
- **db_layer**: Database access, queries, and connection pooling
- **storage_layer**: File storage abstraction (local filesystem, future S3)
- **indexer**: Metadata extraction and cover generation for EPUB/PDF
- **sync_engine**: Synchronization logic and conflict resolution
- **api_server**: HTTP API server (Axum-based routing and middleware)
- **worker_daemon**: Background task processing
- **cli_tool**: Admin CLI

## Project Conventions

- Use `cargo add <crate>` to add new dependencies
- Use `cargo add <crate> --dev` for dev dependencies
- Use `cargo new <crate name>` to add new crates
- Do not manually write version numbers in Cargo.toml

## Design Principles

1. **Interface-first**: Define traits before implementations
2. **Offline-first clients**: Server assumes devices may be offline for extended periods
3. **Content-addressable**: Use hashes for deduplication and integrity
4. **Fail gracefully**: Never corrupt data, prefer safe states
5. **Observable**: Structured logging, metrics, health checks from day one

## Common Development Commands

### Database Setup
```bash
# Start PostgreSQL (Docker)
docker compose up -d postgres

# Set database URL
export DATABASE_URL=postgres://ereader:ereader_dev@localhost/ereader

# Run migrations
sqlx migrate run
```

### Building and Running
```bash
# Run API server locally
cargo run --bin api_server

# Run with hot reload (requires cargo-watch)
cargo watch -x 'run --bin api_server'

# Build all workspace crates
cargo build

# Build for release
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p api_server
cargo test -p db_layer

# Run specific test
cargo test test_name

# Run integration tests only
cargo test --test '*'
```

### SQLx
```bash
# Check SQLx queries at compile time
cargo sqlx prepare -- --lib

# Create new migration
sqlx migrate add migration_name
```

### Docker
```bash
# Start full stack (PostgreSQL + API)
docker compose up

# Start database only
docker compose up -d postgres

# Rebuild and start
docker compose up --build
```

## Key Architecture Patterns

### Error Handling
- Uses `thiserror` for structured errors with context
- Each crate has its own `Error` type that converts to others
- HTTP errors map to appropriate status codes (401 Unauthorized, 404 NotFound, etc.)

### Authentication
- Clerk-based JWT authentication
- Middleware validates tokens and extracts user context
- Device tokens support offline-first clients
- Admin endpoints require additional role checks

### Content Addressing
- Files identified by SHA-256 hash for deduplication
- Original filenames preserved for user experience
- `file_hash` is the primary key for storage operations

### Sync Engine
- Last-write-wins (LWW) conflict resolution using `version` field
- Batch operations for efficiency
- Delta sync to minimize network traffic
- Tombstone records for deletion tracking

### Database Access
- All queries in `db_layer/src/queries/` modules
- SQLx compile-time query verification
- Use `sqlx::query!` and `sqlx::query_as!` macros for type safety
- Connection pooling handled by `PgPool`

### File Storage
- Trait-based abstraction in `storage_layer`
- Current implementation: local filesystem under `/app/data/`
- Structure: `/files/` for ebooks, `/covers/` for thumbnails, `/temp/` for uploads
- Future: S3-compatible storage support

### Logging
- Uses `tracing` for structured logging
- Log levels: trace, debug, info, warn, error
- Include context like `book_id`, `user_id`, `file_size` in log events
- Configure via `RUST_LOG` environment variable

## Configuration

Configuration uses hierarchical loading:
1. Default values in code
2. `config/default.toml`
3. Environment-specific files (`config/production.toml`)
4. Environment variables (prefix: `EREADER__`)

Example: `EREADER__DATABASE__URL=postgres://...`

## Testing Strategy

- Unit tests: In each crate's `tests/` directory or inline with `#[cfg(test)]`
- Integration tests: `crates/api_server/tests/` with shared test utilities
- Test utilities in `tests/common/mod.rs` provide `TestApp` helper
- Use test database with migrations for integration tests
- Mock Clerk authentication in tests

## Important Notes

- The workspace is currently empty (`members = []` in root Cargo.toml) - crates need to be created
- Migrations stored in top-level `migrations/` directory (shared across crates)
- See "Rust API Server Design.md" for complete specifications including database schema, API endpoints, and detailed crate designs
- When implementing sync operations, always consider conflict resolution and use the `version` field
- File uploads should validate format, size, and compute SHA-256 hash before storage
