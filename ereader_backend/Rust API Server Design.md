# E-Reader API Server — Detailed Design Document

## Table of Contents

1. [Overview](#1-overview)
2. [Workspace Structure](#2-workspace-structure)
3. [Crate Designs](#3-crate-designs)
   - [3.1 common](#31-common-crate)
   - [3.2 api_client](#32-api_client-crate)
   - [3.3 db_layer](#33-db_layer-crate)
   - [3.4 storage_layer](#34-storage_layer-crate)
   - [3.5 indexer](#35-indexer-crate)
   - [3.6 sync_engine](#36-sync_engine-crate)
   - [3.7 api_server](#37-api_server-crate)
4. [Database Schema](#4-database-schema)
5. [API Endpoint Specifications](#5-api-endpoint-specifications)
6. [Authentication with Clerk](#6-authentication-with-clerk)
7. [Error Handling Strategy](#7-error-handling-strategy)
8. [Configuration Management](#8-configuration-management)
9. [Docker & Local Development](#9-docker--local-development)
10. [Testing Strategy](#10-testing-strategy)
11. [Observability](#11-observability)
12. [Migration Strategy](#12-migration-strategy)

---

## 1. Overview

### 1.1 Purpose

This document defines the architecture and implementation details for the API server backend of the custom e-reader ecosystem. The server is responsible for:

- Storing and serving ebook files and metadata
- Managing user libraries, collections, and reading progress
- Synchronizing state across multiple devices
- Handling background tasks (indexing, thumbnail generation, etc.)

### 1.2 Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Language | Rust | Performance, safety, single binary deployment |
| Web Framework | Axum | Async, tower ecosystem, excellent ergonomics |
| Database | PostgreSQL | Relational integrity, JSON support, excellent tooling |
| DB Access | SQLx | Compile-time checked queries, async, migrations |
| Authentication | Clerk | Managed auth, device tokens, webhook support |
| Task Queue | tokio + internal queue | Start simple, extract later if needed |
| File Storage | Local filesystem (v1) | Simple, can abstract to S3 later |

### 1.3 Design Principles

1. **Interface-first**: Define traits before implementations
2. **Offline-first clients**: Server assumes devices may be offline for extended periods
3. **Content-addressable**: Use hashes for deduplication and integrity
4. **Fail gracefully**: Never corrupt data, always prefer safe states
5. **Observable**: Structured logging, metrics, health checks from day one

---

## 2. Workspace Structure

```
ereader-server/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── .env.example
├── docker-compose.yml
├── migrations/                   # SQLx migrations (shared)
│   ├── 20240101000000_initial_schema.sql
│   └── ...
├── crates/
│   ├── api_client/              # Rust client SDK for the API
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── client.rs
│   │       ├── error.rs
│   │       ├── auth.rs
│   │       ├── models/
│   │       ├── endpoints/
│   │       │   ├── mod.rs
│   │       │   ├── books.rs
│   │       │   ├── assets.rs
│   │       │   ├── collections.rs
│   │       │   ├── sync.rs
│   │       │   └── admin.rs
│   │       └── offline/
│   │           ├── mod.rs
│   │           ├── cache.rs
│   │           └── queue.rs
│   │
│   ├── api_server/              # HTTP API, routing, request handling
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── routes/
│   │       ├── middleware/
│   │       ├── extractors/
│   │       └── error.rs
│   │
│   ├── db_layer/                # Database access, queries, connection pool
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       ├── queries/
│   │       └── pool.rs
│   │
│   ├── storage_layer/           # File storage abstraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── local.rs
│   │       └── traits.rs
│   │
│   ├── indexer/                 # Metadata extraction, cover generation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── epub.rs
│   │       ├── pdf.rs
│   │       ├── cover.rs
│   │       └── traits.rs
│   │
│   ├── sync_engine/             # Sync logic, conflict resolution
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── merge.rs
│   │       ├── conflicts.rs
│   │       └── batch.rs
│   │
│   ├── worker_daemon/           # Background task processing
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── tasks/
│   │       └── scheduler.rs
│   │
│   ├── cli_tool/                # Admin CLI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/
│   │
│   └── common/                  # Shared types, utilities
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── types.rs
│           ├── config.rs
│           └── error.rs
```

### 2.1 Workspace Cargo.toml

```toml
[workspace]
resolver = "2"
members = [
    "crates/api_client",
    "crates/api_server",
    "crates/db_layer",
    "crates/storage_layer",
    "crates/indexer",
    "crates/sync_engine",
    "crates/worker_daemon",
    "crates/cli_tool",
    "crates/common",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
license = "MIT"
repository = "https://github.com/youruser/ereader-server"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Web framework
axum = { version = "0.7", features = ["macros", "multipart"] }
axum-extra = { version = "0.9", features = ["typed-header"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["cors", "trace", "limit", "compression-gzip"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Auth
jsonwebtoken = "9.2"
reqwest = { version = "0.11", features = ["json", "rustls-tls"], default-features = false }

# Utilities
uuid = { version = "1.6", features = ["v4", "v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
config = "0.14"
dotenvy = "0.15"

# Hashing
sha2 = "0.10"
hex = "0.4"

# Image processing
image = "0.24"

# Ebook processing
epub = "2.1"
lopdf = "0.31"

# CLI
clap = { version = "4.4", features = ["derive"] }

# Testing
tokio-test = "0.4"
```

---

## 3. Crate Designs

### 3.1 `common` Crate

Shared types and utilities used across all crates.

```rust
// crates/common/src/lib.rs
pub mod config;
pub mod error;
pub mod types;

pub use config::AppConfig;
pub use error::{Error, Result};
```

#### 3.1.1 Core Types

```rust
// crates/common/src/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for books (UUID v7 for time-ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct BookId(pub Uuid);

impl BookId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

/// Similarly for other IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub String); // Clerk user ID (string)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct DeviceId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct FileAssetId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct CollectionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct AnnotationId(pub Uuid);

/// Content hash for deduplication and integrity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct ContentHash(pub String); // SHA-256 hex string

impl ContentHash {
    pub fn from_bytes(data: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        Self(hex::encode(hasher.finalize()))
    }
}

/// Reading position - format-agnostic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingLocation {
    /// EPUB CFI, PDF page number, or generic offset
    pub locator: String,
    /// Estimated progress 0.0-1.0
    pub progress: f32,
    /// Optional chapter/section info
    pub chapter: Option<String>,
}

/// Supported ebook formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "book_format", rename_all = "lowercase")]
pub enum BookFormat {
    Epub,
    Pdf,
    Cbz,
    Mobi,
}

impl BookFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "epub" => Some(Self::Epub),
            "pdf" => Some(Self::Pdf),
            "cbz" | "cbr" => Some(Self::Cbz),
            "mobi" | "azw3" => Some(Self::Mobi),
            _ => None,
        }
    }
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Epub => "application/epub+zip",
            Self::Pdf => "application/pdf",
            Self::Cbz => "application/vnd.comicbook+zip",
            Self::Mobi => "application/x-mobipocket-ebook",
        }
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 { 50 }

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
```

#### 3.1.2 Error Types

```rust
// crates/common/src/error.rs
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    // Authentication/Authorization
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("forbidden: {0}")]
    Forbidden(String),
    
    // Resource errors
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("conflict: {0}")]
    Conflict(String),
    
    // Validation
    #[error("validation error: {0}")]
    Validation(String),
    
    // Storage
    #[error("storage error: {0}")]
    Storage(String),
    
    // Database
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    
    // External services
    #[error("external service error: {0}")]
    ExternalService(String),
    
    // Internal
    #[error("internal error: {0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Error::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Error::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            Error::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            Error::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            Error::Validation(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            Error::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "storage_error"),
            Error::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database_error"),
            Error::ExternalService(_) => (StatusCode::BAD_GATEWAY, "external_service_error"),
            Error::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        
        let body = ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message: self.to_string(),
                details: None,
            },
        };
        
        (status, Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
```

#### 3.1.3 Configuration

```rust
// crates/common/src/config.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub clerk: ClerkConfig,
    pub worker: WorkerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_body_limit: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".into(),
            port: 8080,
            request_body_limit: 100 * 1024 * 1024, // 100MB
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub base_path: String,
    pub covers_path: String,
    pub temp_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClerkConfig {
    pub publishable_key: String,
    pub secret_key: String,
    pub jwks_url: String,
    pub webhook_secret: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    pub concurrency: usize,
    pub poll_interval_ms: u64,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("EREADER").separator("__"))
            .build()?;
        
        Ok(config.try_deserialize()?)
    }
}
```

---

### 3.2 `api_client` Crate

The API client is a first-class citizen of the workspace, designed for use by the e-reader device, desktop applications, CLI tools, and any other Rust application that needs to communicate with the server.

#### 3.2.1 Design Goals

1. **Ergonomic API**: Builder pattern for requests, sensible defaults
2. **Offline-first**: Request queuing, response caching, optimistic updates
3. **Type-safe**: Shared types with server, compile-time guarantees
4. **Flexible auth**: Support Clerk JWTs, device tokens, and token refresh
5. **Observable**: Hooks for logging, metrics, progress callbacks
6. **Portable**: Works on desktop, embedded Linux (e-reader), and WASM (future)

#### 3.2.2 Crate Structure

```rust
// crates/api_client/src/lib.rs
pub mod auth;
pub mod client;
pub mod endpoints;
pub mod error;
pub mod models;
pub mod offline;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use models::*;
```

#### 3.2.3 Client Builder & Configuration

```rust
// crates/api_client/src/client.rs
use crate::auth::{AuthProvider, TokenAuth};
use crate::endpoints::{AdminEndpoint, AssetsEndpoint, BooksEndpoint, CollectionsEndpoint, SyncEndpoint};
use crate::error::{Error, Result};
use crate::offline::{OfflineQueue, ResponseCache};
use reqwest::{Client as HttpClient, ClientBuilder as HttpClientBuilder};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Configuration for the API client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub connect_timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub user_agent: String,
    pub enable_compression: bool,
    pub cache_dir: Option<std::path::PathBuf>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".into(),
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
            user_agent: format!("ereader-client/{}", env!("CARGO_PKG_VERSION")),
            enable_compression: true,
            cache_dir: None,
        }
    }
}

/// Builder for creating configured API clients
pub struct ClientBuilder {
    config: ClientConfig,
    auth: Option<Arc<dyn AuthProvider>>,
    offline_mode: bool,
}

impl ClientBuilder {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            config: ClientConfig {
                base_url: base_url.into(),
                ..Default::default()
            },
            auth: None,
            offline_mode: false,
        }
    }
    
    /// Set authentication provider
    pub fn with_auth(mut self, auth: impl AuthProvider + 'static) -> Self {
        self.auth = Some(Arc::new(auth));
        self
    }
    
    /// Set a static bearer token (convenience method)
    pub fn with_token(self, token: impl Into<String>) -> Self {
        self.with_auth(TokenAuth::new(token))
    }
    
    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }
    
    /// Set connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }
    
    /// Set maximum retry attempts for failed requests
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }
    
    /// Set custom user agent
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.user_agent = agent.into();
        self
    }
    
    /// Enable offline mode with caching and request queuing
    pub fn enable_offline(mut self, cache_dir: impl Into<std::path::PathBuf>) -> Self {
        self.offline_mode = true;
        self.config.cache_dir = Some(cache_dir.into());
        self
    }
    
    /// Build the client
    pub fn build(self) -> Result<Client> {
        let http = HttpClientBuilder::new()
            .timeout(self.config.timeout)
            .connect_timeout(self.config.connect_timeout)
            .user_agent(&self.config.user_agent)
            .gzip(self.config.enable_compression)
            .build()
            .map_err(|e| Error::Configuration(e.to_string()))?;
        
        let cache = if self.offline_mode {
            self.config.cache_dir.as_ref().map(|dir| {
                Arc::new(RwLock::new(ResponseCache::new(dir)))
            })
        } else {
            None
        };
        
        let queue = if self.offline_mode {
            self.config.cache_dir.as_ref().map(|dir| {
                Arc::new(RwLock::new(OfflineQueue::new(dir)))
            })
        } else {
            None
        };
        
        Ok(Client {
            http,
            config: self.config,
            auth: self.auth,
            cache,
            queue,
        })
    }
}

/// The main API client
#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    config: ClientConfig,
    auth: Option<Arc<dyn AuthProvider>>,
    cache: Option<Arc<RwLock<ResponseCache>>>,
    queue: Option<Arc<RwLock<OfflineQueue>>>,
}

impl Client {
    /// Create a new client builder
    pub fn builder(base_url: impl Into<String>) -> ClientBuilder {
        ClientBuilder::new(base_url)
    }
    
    /// Quick constructor with just a base URL
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        ClientBuilder::new(base_url).build()
    }
    
    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }
    
    /// Check if the server is reachable
    pub async fn health_check(&self) -> Result<bool> {
        let response = self.http
            .get(format!("{}/api/v1/health", self.config.base_url))
            .send()
            .await;
        
        Ok(response.map(|r| r.status().is_success()).unwrap_or(false))
    }
    
    /// Books endpoint
    pub fn books(&self) -> BooksEndpoint {
        BooksEndpoint::new(self.clone())
    }
    
    /// Assets endpoint
    pub fn assets(&self) -> AssetsEndpoint {
        AssetsEndpoint::new(self.clone())
    }
    
    /// Collections endpoint
    pub fn collections(&self) -> CollectionsEndpoint {
        CollectionsEndpoint::new(self.clone())
    }
    
    /// Sync endpoint
    pub fn sync(&self) -> SyncEndpoint {
        SyncEndpoint::new(self.clone())
    }
    
    /// Admin endpoint
    pub fn admin(&self) -> AdminEndpoint {
        AdminEndpoint::new(self.clone())
    }
    
    /// Process any queued offline requests
    pub async fn flush_offline_queue(&self) -> Result<QueueFlushResult> {
        let queue = self.queue.as_ref()
            .ok_or_else(|| Error::Configuration("Offline mode not enabled".into()))?;
        
        let mut queue = queue.write().await;
        queue.flush(self).await
    }
    
    // Internal request method with auth, retries, and caching
    pub(crate) async fn request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<T> {
        self.request_with_options(method, path, body, RequestOptions::default()).await
    }
    
    pub(crate) async fn request_with_options<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<&impl serde::Serialize>,
        options: RequestOptions,
    ) -> Result<T> {
        let url = format!("{}{}", self.config.base_url, path);
        
        // Check cache for GET requests
        if method == reqwest::Method::GET && options.use_cache {
            if let Some(cache) = &self.cache {
                let cache = cache.read().await;
                if let Some(cached) = cache.get(&url) {
                    return serde_json::from_slice(&cached)
                        .map_err(|e| Error::Deserialization(e.to_string()));
                }
            }
        }
        
        let mut attempts = 0;
        let max_attempts = if options.retry { self.config.max_retries + 1 } else { 1 };
        
        loop {
            attempts += 1;
            
            let mut request = self.http.request(method.clone(), &url);
            
            // Add auth header
            if let Some(auth) = &self.auth {
                let token = auth.get_token().await?;
                request = request.header("Authorization", format!("Bearer {}", token));
            }
            
            // Add body
            if let Some(b) = body {
                request = request.json(b);
            }
            
            let result = request.send().await;
            
            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        let bytes = response.bytes().await
                            .map_err(|e| Error::Network(e.to_string()))?;
                        
                        // Cache successful GET responses
                        if method == reqwest::Method::GET && options.cache_response {
                            if let Some(cache) = &self.cache {
                                let mut cache = cache.write().await;
                                cache.set(&url, &bytes, options.cache_ttl);
                            }
                        }
                        
                        return serde_json::from_slice(&bytes)
                            .map_err(|e| Error::Deserialization(e.to_string()));
                    } else if response.status().as_u16() == 401 {
                        // Try to refresh token
                        if let Some(auth) = &self.auth {
                            if auth.refresh().await.is_ok() && attempts < max_attempts {
                                continue;
                            }
                        }
                        return Err(Error::Unauthorized);
                    } else if response.status().as_u16() == 404 {
                        return Err(Error::NotFound);
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        return Err(Error::Api { status: status.as_u16(), message: body });
                    }
                }
                Err(e) => {
                    if attempts >= max_attempts {
                        // Queue for offline if enabled
                        if options.queue_on_failure {
                            if let Some(queue) = &self.queue {
                                let mut queue = queue.write().await;
                                queue.enqueue(QueuedRequest {
                                    method: method.to_string(),
                                    path: path.to_string(),
                                    body: body.map(|b| serde_json::to_vec(b).ok()).flatten(),
                                })?;
                            }
                        }
                        return Err(Error::Network(e.to_string()));
                    }
                    
                    tokio::time::sleep(self.config.retry_delay * attempts).await;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RequestOptions {
    pub retry: bool,
    pub use_cache: bool,
    pub cache_response: bool,
    pub cache_ttl: Option<Duration>,
    pub queue_on_failure: bool,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self {
            retry: true,
            use_cache: true,
            cache_response: true,
            cache_ttl: None,
            queue_on_failure: false,
        }
    }
}

#[derive(Debug)]
pub struct QueueFlushResult {
    pub succeeded: usize,
    pub failed: usize,
    pub remaining: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueuedRequest {
    pub method: String,
    pub path: String,
    pub body: Option<Vec<u8>>,
}
```

#### 3.2.4 Authentication Providers

```rust
// crates/api_client/src/auth.rs
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Trait for authentication providers
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Get the current authentication token
    async fn get_token(&self) -> Result<String>;
    
    /// Refresh the token if possible
    async fn refresh(&self) -> Result<()>;
    
    /// Check if the token is expired
    async fn is_expired(&self) -> bool;
}

/// Simple static token authentication
pub struct TokenAuth {
    token: String,
}

impl TokenAuth {
    pub fn new(token: impl Into<String>) -> Self {
        Self { token: token.into() }
    }
}

#[async_trait]
impl AuthProvider for TokenAuth {
    async fn get_token(&self) -> Result<String> {
        Ok(self.token.clone())
    }
    
    async fn refresh(&self) -> Result<()> {
        // Static tokens can't be refreshed
        Err(Error::Unauthorized)
    }
    
    async fn is_expired(&self) -> bool {
        false // Static tokens don't expire (from client's perspective)
    }
}

/// Clerk JWT authentication with automatic refresh
pub struct ClerkAuth {
    session_token: Arc<RwLock<String>>,
    refresh_token: Arc<RwLock<Option<String>>>,
    clerk_frontend_api: String,
    expiry: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl ClerkAuth {
    pub fn new(
        session_token: impl Into<String>,
        refresh_token: Option<String>,
        clerk_frontend_api: impl Into<String>,
    ) -> Self {
        Self {
            session_token: Arc::new(RwLock::new(session_token.into())),
            refresh_token: Arc::new(RwLock::new(refresh_token)),
            clerk_frontend_api: clerk_frontend_api.into(),
            expiry: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Update tokens (called after successful refresh or re-authentication)
    pub async fn update_tokens(&self, session_token: String, refresh_token: Option<String>) {
        *self.session_token.write().await = session_token;
        *self.refresh_token.write().await = refresh_token;
    }
}

#[async_trait]
impl AuthProvider for ClerkAuth {
    async fn get_token(&self) -> Result<String> {
        // Check if we need to refresh
        if self.is_expired().await {
            self.refresh().await?;
        }
        Ok(self.session_token.read().await.clone())
    }
    
    async fn refresh(&self) -> Result<()> {
        let refresh_token = self.refresh_token.read().await.clone()
            .ok_or(Error::Unauthorized)?;
        
        // Call Clerk's token refresh endpoint
        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/v1/client/sessions/refresh", self.clerk_frontend_api))
            .header("Authorization", format!("Bearer {}", refresh_token))
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(Error::Unauthorized);
        }
        
        #[derive(serde::Deserialize)]
        struct RefreshResponse {
            session_token: String,
            #[serde(default)]
            refresh_token: Option<String>,
        }
        
        let data: RefreshResponse = response.json().await
            .map_err(|e| Error::Deserialization(e.to_string()))?;
        
        self.update_tokens(data.session_token, data.refresh_token).await;
        
        Ok(())
    }
    
    async fn is_expired(&self) -> bool {
        if let Some(expiry) = *self.expiry.read().await {
            expiry <= chrono::Utc::now()
        } else {
            false
        }
    }
}

/// Device-based authentication for e-reader devices
pub struct DeviceAuth {
    device_id: uuid::Uuid,
    device_token: Arc<RwLock<String>>,
    server_url: String,
}

impl DeviceAuth {
    pub fn new(
        device_id: uuid::Uuid,
        device_token: impl Into<String>,
        server_url: impl Into<String>,
    ) -> Self {
        Self {
            device_id,
            device_token: Arc::new(RwLock::new(device_token.into())),
            server_url: server_url.into(),
        }
    }
    
    pub fn device_id(&self) -> uuid::Uuid {
        self.device_id
    }
}

#[async_trait]
impl AuthProvider for DeviceAuth {
    async fn get_token(&self) -> Result<String> {
        Ok(self.device_token.read().await.clone())
    }
    
    async fn refresh(&self) -> Result<()> {
        // Device tokens are long-lived, refresh via re-pairing if needed
        Err(Error::Unauthorized)
    }
    
    async fn is_expired(&self) -> bool {
        false
    }
}
```

#### 3.2.5 Client Error Types

```rust
// crates/api_client/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("network error: {0}")]
    Network(String),
    
    #[error("unauthorized")]
    Unauthorized,
    
    #[error("not found")]
    NotFound,
    
    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },
    
    #[error("serialization error: {0}")]
    Serialization(String),
    
    #[error("deserialization error: {0}")]
    Deserialization(String),
    
    #[error("configuration error: {0}")]
    Configuration(String),
    
    #[error("offline: {0}")]
    Offline(String),
    
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self, Error::Network(_) | Error::Api { status: 500..=599, .. })
    }
    
    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(self, 
            Error::Unauthorized | 
            Error::NotFound | 
            Error::Validation(_) |
            Error::Api { status: 400..=499, .. }
        )
    }
}
```

#### 3.2.6 Shared Models

```rust
// crates/api_client/src/models/mod.rs
pub mod book;
pub mod collection;
pub mod sync;

pub use book::*;
pub use collection::*;
pub use sync::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Paginated response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> Paginated<T> {
    pub fn has_more(&self) -> bool {
        self.offset + self.items.len() as i64 < self.total
    }
    
    pub fn next_offset(&self) -> i64 {
        self.offset + self.items.len() as i64
    }
}

/// Standard timestamp fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamps {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

```rust
// crates/api_client/src/models/book.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookFormat {
    Epub,
    Pdf,
    Cbz,
    Mobi,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Book {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BookDetail {
    #[serde(flatten)]
    pub book: Book,
    pub formats: Vec<FormatInfo>,
    pub covers: Vec<CoverInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormatInfo {
    pub format: BookFormat,
    pub file_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoverInfo {
    pub size: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateBookRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_index: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateBookRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series_index: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadResponse {
    pub asset_id: Uuid,
    pub format: BookFormat,
    pub file_size: i64,
    pub content_hash: String,
}
```

```rust
// crates/api_client/src/models/sync.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingLocation {
    pub locator: String,
    pub progress: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingStateSync {
    pub book_id: Uuid,
    pub location: ReadingLocation,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationSync {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub book_id: Uuid,
    pub annotation_type: AnnotationType,
    pub location_start: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncRequest {
    pub device_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reading_states: Vec<ReadingStateSync>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<AnnotationSync>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncResponse {
    pub server_time: chrono::DateTime<chrono::Utc>,
    pub reading_states: Vec<ReadingStateSync>,
    pub annotations: Vec<AnnotationSync>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SyncConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub local_updated_at: chrono::DateTime<chrono::Utc>,
    pub server_updated_at: chrono::DateTime<chrono::Utc>,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Merged,
}
```

#### 3.2.7 Endpoint Implementations

```rust
// crates/api_client/src/endpoints/mod.rs
pub mod admin;
pub mod assets;
pub mod books;
pub mod collections;
pub mod sync;

pub use admin::AdminEndpoint;
pub use assets::AssetsEndpoint;
pub use books::BooksEndpoint;
pub use collections::CollectionsEndpoint;
pub use sync::SyncEndpoint;
```

```rust
// crates/api_client/src/endpoints/books.rs
use crate::client::Client;
use crate::error::Result;
use crate::models::{Book, BookDetail, CreateBookRequest, Paginated, UpdateBookRequest};
use uuid::Uuid;

/// Books API endpoint
pub struct BooksEndpoint {
    client: Client,
}

impl BooksEndpoint {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }
    
    /// List books with pagination
    pub fn list(&self) -> ListBooksBuilder {
        ListBooksBuilder::new(self.client.clone())
    }
    
    /// Search books
    pub fn search(&self, query: impl Into<String>) -> SearchBooksBuilder {
        SearchBooksBuilder::new(self.client.clone(), query.into())
    }
    
    /// Get a single book by ID
    pub async fn get(&self, id: Uuid) -> Result<BookDetail> {
        self.client.request(
            reqwest::Method::GET,
            &format!("/api/v1/books/{}", id),
            None::<&()>,
        ).await
    }
    
    /// Create a new book
    pub fn create(&self) -> CreateBookBuilder {
        CreateBookBuilder::new(self.client.clone())
    }
    
    /// Update an existing book
    pub fn update(&self, id: Uuid) -> UpdateBookBuilder {
        UpdateBookBuilder::new(self.client.clone(), id)
    }
    
    /// Delete a book
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        self.client.request::<serde_json::Value>(
            reqwest::Method::DELETE,
            &format!("/api/v1/books/{}", id),
            None::<&()>,
        ).await?;
        Ok(())
    }
}

/// Builder for listing books
pub struct ListBooksBuilder {
    client: Client,
    limit: i64,
    offset: i64,
    sort_by: Option<String>,
    sort_order: Option<String>,
    tag: Option<String>,
    series: Option<String>,
}

impl ListBooksBuilder {
    fn new(client: Client) -> Self {
        Self {
            client,
            limit: 50,
            offset: 0,
            sort_by: None,
            sort_order: None,
            tag: None,
            series: None,
        }
    }
    
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }
    
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }
    
    pub fn sort_order(mut self, order: impl Into<String>) -> Self {
        self.sort_order = Some(order.into());
        self
    }
    
    pub fn descending(self) -> Self {
        self.sort_order("desc")
    }
    
    pub fn ascending(self) -> Self {
        self.sort_order("asc")
    }
    
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }
    
    pub fn series(mut self, series: impl Into<String>) -> Self {
        self.series = Some(series.into());
        self
    }
    
    pub async fn send(self) -> Result<Paginated<Book>> {
        let mut params = vec![
            ("limit", self.limit.to_string()),
            ("offset", self.offset.to_string()),
        ];
        
        if let Some(sort_by) = self.sort_by {
            params.push(("sort_by", sort_by));
        }
        if let Some(sort_order) = self.sort_order {
            params.push(("sort_order", sort_order));
        }
        if let Some(tag) = self.tag {
            params.push(("tag", tag));
        }
        if let Some(series) = self.series {
            params.push(("series", series));
        }
        
        let query = serde_urlencoded::to_string(&params)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
        
        self.client.request(
            reqwest::Method::GET,
            &format!("/api/v1/books?{}", query),
            None::<&()>,
        ).await
    }
}

/// Builder for searching books
pub struct SearchBooksBuilder {
    client: Client,
    query: String,
    limit: i64,
    offset: i64,
}

impl SearchBooksBuilder {
    fn new(client: Client, query: String) -> Self {
        Self {
            client,
            query,
            limit: 50,
            offset: 0,
        }
    }
    
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = limit;
        self
    }
    
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }
    
    pub async fn send(self) -> Result<Paginated<Book>> {
        let params = [
            ("q", self.query),
            ("limit", self.limit.to_string()),
            ("offset", self.offset.to_string()),
        ];
        
        let query = serde_urlencoded::to_string(&params)
            .map_err(|e| crate::error::Error::Serialization(e.to_string()))?;
        
        self.client.request(
            reqwest::Method::GET,
            &format!("/api/v1/books/search?{}", query),
            None::<&()>,
        ).await
    }
}

/// Builder for creating books
pub struct CreateBookBuilder {
    client: Client,
    request: CreateBookRequest,
}

impl CreateBookBuilder {
    fn new(client: Client) -> Self {
        Self {
            client,
            request: CreateBookRequest::default(),
        }
    }
    
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = title.into();
        self
    }
    
    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.request.authors = Some(authors);
        self
    }
    
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.request.authors = Some(vec![author.into()]);
        self
    }
    
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.request.description = Some(description.into());
        self
    }
    
    pub fn series(mut self, name: impl Into<String>, index: f32) -> Self {
        self.request.series_name = Some(name.into());
        self.request.series_index = Some(index);
        self
    }
    
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.request.tags = Some(tags);
        self
    }
    
    pub async fn send(self) -> Result<Book> {
        self.client.request(
            reqwest::Method::POST,
            "/api/v1/books",
            Some(&self.request),
        ).await
    }
}

/// Builder for updating books
pub struct UpdateBookBuilder {
    client: Client,
    id: Uuid,
    request: UpdateBookRequest,
}

impl UpdateBookBuilder {
    fn new(client: Client, id: Uuid) -> Self {
        Self {
            client,
            id,
            request: UpdateBookRequest::default(),
        }
    }
    
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.request.title = Some(title.into());
        self
    }
    
    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.request.authors = Some(authors);
        self
    }
    
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.request.description = Some(description.into());
        self
    }
    
    pub fn series(mut self, name: impl Into<String>, index: f32) -> Self {
        self.request.series_name = Some(name.into());
        self.request.series_index = Some(index);
        self
    }
    
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.request.tags = Some(tags);
        self
    }
    
    pub async fn send(self) -> Result<Book> {
        self.client.request(
            reqwest::Method::PUT,
            &format!("/api/v1/books/{}", self.id),
            Some(&self.request),
        ).await
    }
}
```

```rust
// crates/api_client/src/endpoints/assets.rs
use crate::client::Client;
use crate::error::{Error, Result};
use crate::models::{BookFormat, UploadResponse};
use std::path::Path;
use tokio::io::AsyncRead;
use uuid::Uuid;

/// Assets API endpoint for file uploads/downloads
pub struct AssetsEndpoint {
    client: Client,
}

impl AssetsEndpoint {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }
    
    /// Upload a file from a path
    pub async fn upload_file(&self, book_id: Uuid, path: impl AsRef<Path>) -> Result<UploadResponse> {
        let path = path.as_ref();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::Validation("Invalid filename".into()))?
            .to_string();
        
        let data = tokio::fs::read(path).await?;
        self.upload_bytes(book_id, &filename, data).await
    }
    
    /// Upload a file from bytes
    pub async fn upload_bytes(
        &self,
        book_id: Uuid,
        filename: &str,
        data: Vec<u8>,
    ) -> Result<UploadResponse> {
        let url = format!("{}/api/v1/books/{}/upload", self.client.base_url(), book_id);
        
        let part = reqwest::multipart::Part::bytes(data)
            .file_name(filename.to_string());
        
        let form = reqwest::multipart::Form::new()
            .part("file", part);
        
        let mut request = reqwest::Client::new()
            .post(&url)
            .multipart(form);
        
        // Add auth - we need to access internal client state
        // This is a simplified version; real impl would use the auth provider
        
        let response = request.send().await
            .map_err(|e| Error::Network(e.to_string()))?;
        
        if response.status().is_success() {
            response.json().await
                .map_err(|e| Error::Deserialization(e.to_string()))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::Api { status, message })
        }
    }
    
    /// Download a book file
    pub async fn download(&self, book_id: Uuid) -> Result<Vec<u8>> {
        self.download_format(book_id, None).await
    }
    
    /// Download a specific format
    pub async fn download_format(&self, book_id: Uuid, format: Option<BookFormat>) -> Result<Vec<u8>> {
        let path = match format {
            Some(fmt) => format!("/api/v1/books/{}/download/{:?}", book_id, fmt).to_lowercase(),
            None => format!("/api/v1/books/{}/download", book_id),
        };
        
        let url = format!("{}{}", self.client.base_url(), path);
        
        let response = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;
        
        if response.status().is_success() {
            response.bytes().await
                .map(|b| b.to_vec())
                .map_err(|e| Error::Network(e.to_string()))
        } else if response.status().as_u16() == 404 {
            Err(Error::NotFound)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::Api { status, message })
        }
    }
    
    /// Download to a file
    pub async fn download_to_file(
        &self,
        book_id: Uuid,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let data = self.download(book_id).await?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }
    
    /// Get cover image
    pub async fn get_cover(&self, book_id: Uuid, size: Option<&str>) -> Result<Vec<u8>> {
        let path = match size {
            Some(s) => format!("/api/v1/books/{}/cover/{}", book_id, s),
            None => format!("/api/v1/books/{}/cover", book_id),
        };
        
        let url = format!("{}{}", self.client.base_url(), path);
        
        let response = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .map_err(|e| Error::Network(e.to_string()))?;
        
        if response.status().is_success() {
            response.bytes().await
                .map(|b| b.to_vec())
                .map_err(|e| Error::Network(e.to_string()))
        } else {
            Err(Error::NotFound)
        }
    }
}
```

```rust
// crates/api_client/src/endpoints/sync.rs
use crate::client::Client;
use crate::error::Result;
use crate::models::{
    AnnotationSync, AnnotationType, ReadingLocation, ReadingStateSync, 
    SyncRequest, SyncResponse,
};
use uuid::Uuid;

/// Sync API endpoint
pub struct SyncEndpoint {
    client: Client,
}

impl SyncEndpoint {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }
    
    /// Perform a batch sync
    pub fn batch(&self, device_id: Uuid) -> SyncBatchBuilder {
        SyncBatchBuilder::new(self.client.clone(), device_id)
    }
    
    /// Get reading state for a book
    pub async fn get_reading_state(&self, book_id: Uuid) -> Result<Option<ReadingStateSync>> {
        self.client.request(
            reqwest::Method::GET,
            &format!("/api/v1/sync/reading-state/{}", book_id),
            None::<&()>,
        ).await
    }
    
    /// Update reading state for a book
    pub async fn update_reading_state(
        &self,
        book_id: Uuid,
        device_id: Uuid,
        location: ReadingLocation,
    ) -> Result<ReadingStateSync> {
        #[derive(serde::Serialize)]
        struct Request {
            device_id: Uuid,
            location: ReadingLocation,
        }
        
        self.client.request(
            reqwest::Method::PUT,
            &format!("/api/v1/sync/reading-state/{}", book_id),
            Some(&Request { device_id, location }),
        ).await
    }
    
    /// Get all annotations
    pub async fn list_annotations(&self) -> Result<Vec<AnnotationSync>> {
        self.client.request(
            reqwest::Method::GET,
            "/api/v1/sync/annotations",
            None::<&()>,
        ).await
    }
    
    /// Get annotations for a book
    pub async fn get_book_annotations(&self, book_id: Uuid) -> Result<Vec<AnnotationSync>> {
        self.client.request(
            reqwest::Method::GET,
            &format!("/api/v1/sync/annotations/{}", book_id),
            None::<&()>,
        ).await
    }
}

/// Builder for batch sync requests
pub struct SyncBatchBuilder {
    client: Client,
    request: SyncRequest,
}

impl SyncBatchBuilder {
    fn new(client: Client, device_id: Uuid) -> Self {
        Self {
            client,
            request: SyncRequest {
                device_id,
                ..Default::default()
            },
        }
    }
    
    /// Set last sync timestamp
    pub fn since(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.request.last_sync_at = Some(timestamp);
        self
    }
    
    /// Add a reading state to sync
    pub fn reading_state(mut self, state: ReadingStateSync) -> Self {
        self.request.reading_states.push(state);
        self
    }
    
    /// Add multiple reading states
    pub fn reading_states(mut self, states: impl IntoIterator<Item = ReadingStateSync>) -> Self {
        self.request.reading_states.extend(states);
        self
    }
    
    /// Add an annotation to sync
    pub fn annotation(mut self, annotation: AnnotationSync) -> Self {
        self.request.annotations.push(annotation);
        self
    }
    
    /// Add multiple annotations
    pub fn annotations(mut self, annotations: impl IntoIterator<Item = AnnotationSync>) -> Self {
        self.request.annotations.extend(annotations);
        self
    }
    
    /// Execute the sync
    pub async fn send(self) -> Result<SyncResponse> {
        self.client.request(
            reqwest::Method::POST,
            "/api/v1/sync",
            Some(&self.request),
        ).await
    }
}
```

#### 3.2.8 Offline Support

```rust
// crates/api_client/src/offline/mod.rs
pub mod cache;
pub mod queue;

pub use cache::ResponseCache;
pub use queue::OfflineQueue;
```

```rust
// crates/api_client/src/offline/cache.rs
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Simple response cache for offline support
pub struct ResponseCache {
    cache_dir: PathBuf,
    memory_cache: HashMap<String, CachedResponse>,
    default_ttl: Duration,
}

struct CachedResponse {
    data: Vec<u8>,
    cached_at: Instant,
    ttl: Duration,
}

impl ResponseCache {
    pub fn new(cache_dir: impl AsRef<Path>) -> Self {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&cache_dir).ok();
        
        Self {
            cache_dir,
            memory_cache: HashMap::new(),
            default_ttl: Duration::from_secs(300), // 5 minutes
        }
    }
    
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Check memory cache first
        if let Some(cached) = self.memory_cache.get(key) {
            if cached.cached_at.elapsed() < cached.ttl {
                return Some(cached.data.clone());
            }
        }
        
        // Check disk cache
        let hash = self.hash_key(key);
        let path = self.cache_dir.join(&hash);
        
        if path.exists() {
            if let Ok(data) = std::fs::read(&path) {
                return Some(data);
            }
        }
        
        None
    }
    
    pub fn set(&mut self, key: &str, data: &[u8], ttl: Option<Duration>) {
        let ttl = ttl.unwrap_or(self.default_ttl);
        
        // Store in memory
        self.memory_cache.insert(key.to_string(), CachedResponse {
            data: data.to_vec(),
            cached_at: Instant::now(),
            ttl,
        });
        
        // Store on disk
        let hash = self.hash_key(key);
        let path = self.cache_dir.join(&hash);
        let _ = std::fs::write(&path, data);
    }
    
    pub fn invalidate(&mut self, key: &str) {
        self.memory_cache.remove(key);
        
        let hash = self.hash_key(key);
        let path = self.cache_dir.join(&hash);
        let _ = std::fs::remove_file(&path);
    }
    
    pub fn clear(&mut self) {
        self.memory_cache.clear();
        
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
    
    fn hash_key(&self, key: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}
```

```rust
// crates/api_client/src/offline/queue.rs
use crate::client::{Client, QueueFlushResult, QueuedRequest};
use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Queue for offline requests that will be sent when connectivity returns
pub struct OfflineQueue {
    queue_dir: PathBuf,
    requests: Vec<QueuedRequest>,
}

impl OfflineQueue {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let queue_dir = base_dir.as_ref().join("queue");
        std::fs::create_dir_all(&queue_dir).ok();
        
        // Load existing queued requests
        let requests = Self::load_from_disk(&queue_dir);
        
        Self { queue_dir, requests }
    }
    
    fn load_from_disk(dir: &Path) -> Vec<QueuedRequest> {
        let mut requests = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(data) = std::fs::read(entry.path()) {
                    if let Ok(req) = serde_json::from_slice(&data) {
                        requests.push(req);
                    }
                }
            }
        }
        
        requests
    }
    
    pub fn enqueue(&mut self, request: QueuedRequest) -> Result<()> {
        // Save to disk
        let id = uuid::Uuid::new_v4();
        let path = self.queue_dir.join(format!("{}.json", id));
        let data = serde_json::to_vec(&request)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        std::fs::write(&path, &data)?;
        
        self.requests.push(request);
        Ok(())
    }
    
    pub fn len(&self) -> usize {
        self.requests.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
    
    pub async fn flush(&mut self, client: &Client) -> Result<QueueFlushResult> {
        let mut succeeded = 0;
        let mut failed = 0;
        let mut remaining_requests = Vec::new();
        
        for request in self.requests.drain(..) {
            let method = request.method.parse()
                .unwrap_or(reqwest::Method::POST);
            
            let result: std::result::Result<serde_json::Value, _> = if let Some(body) = &request.body {
                // Has body
                let body: serde_json::Value = serde_json::from_slice(body)
                    .unwrap_or(serde_json::Value::Null);
                client.request(method, &request.path, Some(&body)).await
            } else {
                client.request(method, &request.path, None::<&()>).await
            };
            
            match result {
                Ok(_) => succeeded += 1,
                Err(e) if e.is_retryable() => {
                    remaining_requests.push(request);
                    failed += 1;
                }
                Err(_) => failed += 1, // Non-retryable, discard
            }
        }
        
        // Re-queue failed retryable requests
        let remaining = remaining_requests.len();
        for req in remaining_requests {
            self.enqueue(req)?;
        }
        
        // Clean up disk
        self.cleanup_disk()?;
        
        Ok(QueueFlushResult {
            succeeded,
            failed,
            remaining,
        })
    }
    
    fn cleanup_disk(&self) -> Result<()> {
        // Remove all files and re-persist current queue
        if let Ok(entries) = std::fs::read_dir(&self.queue_dir) {
            for entry in entries.flatten() {
                let _ = std::fs::remove_file(entry.path());
            }
        }
        
        for (i, req) in self.requests.iter().enumerate() {
            let path = self.queue_dir.join(format!("{}.json", i));
            let data = serde_json::to_vec(req)
                .map_err(|e| Error::Serialization(e.to_string()))?;
            std::fs::write(&path, &data)?;
        }
        
        Ok(())
    }
}
```

#### 3.2.9 Client Usage Examples

```rust
// Example: Basic usage
use api_client::{Client, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Create client with token auth
    let client = Client::builder("https://api.example.com")
        .with_token("your-clerk-jwt")
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    // Check health
    if !client.health_check().await? {
        eprintln!("Server is not reachable");
        return Ok(());
    }
    
    // List recent books
    let books = client.books()
        .list()
        .limit(20)
        .sort_by("created_at")
        .descending()
        .send()
        .await?;
    
    println!("Found {} books", books.total);
    for book in &books.items {
        println!("  - {} by {:?}", book.title, book.authors);
    }
    
    // Create a new book
    let new_book = client.books()
        .create()
        .title("The Rust Programming Language")
        .authors(vec!["Steve Klabnik".into(), "Carol Nichols".into()])
        .tags(vec!["programming".into(), "rust".into()])
        .send()
        .await?;
    
    println!("Created book: {}", new_book.id);
    
    // Upload a file
    let upload_result = client.assets()
        .upload_file(new_book.id, "book.epub")
        .await?;
    
    println!("Uploaded: {} bytes, hash: {}", upload_result.file_size, upload_result.content_hash);
    
    Ok(())
}
```

```rust
// Example: E-reader device with offline support
use api_client::{Client, DeviceAuth, Result};
use api_client::models::{ReadingLocation, ReadingStateSync, SyncRequest};
use uuid::Uuid;

struct EReaderApp {
    client: Client,
    device_id: Uuid,
    last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

impl EReaderApp {
    async fn new(server_url: &str, device_id: Uuid, device_token: &str) -> Result<Self> {
        let auth = DeviceAuth::new(device_id, device_token, server_url);
        
        let client = Client::builder(server_url)
            .with_auth(auth)
            .enable_offline("/data/cache")
            .max_retries(5)
            .build()?;
        
        Ok(Self {
            client,
            device_id,
            last_sync: None,
        })
    }
    
    async fn sync(&mut self, local_states: Vec<ReadingStateSync>) -> Result<()> {
        let response = self.client.sync()
            .batch(self.device_id)
            .since(self.last_sync.unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(30)))
            .reading_states(local_states)
            .send()
            .await?;
        
        self.last_sync = Some(response.server_time);
        
        // Handle conflicts
        for conflict in &response.conflicts {
            println!("Conflict on {}: {:?}", conflict.entity_id, conflict.resolution);
        }
        
        // Apply server states locally
        for state in response.reading_states {
            self.apply_reading_state(state).await?;
        }
        
        Ok(())
    }
    
    async fn update_progress(&self, book_id: Uuid, progress: f32, locator: &str) -> Result<()> {
        let location = ReadingLocation {
            locator: locator.to_string(),
            progress,
            chapter: None,
        };
        
        self.client.sync()
            .update_reading_state(book_id, self.device_id, location)
            .await?;
        
        Ok(())
    }
    
    async fn download_book(&self, book_id: Uuid, local_path: &str) -> Result<()> {
        self.client.assets()
            .download_to_file(book_id, local_path)
            .await
    }
    
    async fn apply_reading_state(&self, _state: ReadingStateSync) -> Result<()> {
        // Apply to local database
        Ok(())
    }
    
    async fn flush_offline_queue(&self) -> Result<()> {
        let result = self.client.flush_offline_queue().await?;
        println!(
            "Flushed queue: {} succeeded, {} failed, {} remaining",
            result.succeeded, result.failed, result.remaining
        );
        Ok(())
    }
}
```

```rust
// Example: Pagination helper
use api_client::{Client, Result};
use api_client::models::Book;

async fn fetch_all_books(client: &Client) -> Result<Vec<Book>> {
    let mut all_books = Vec::new();
    let mut offset = 0;
    let limit = 100;
    
    loop {
        let page = client.books()
            .list()
            .limit(limit)
            .offset(offset)
            .send()
            .await?;
        
        all_books.extend(page.items);
        
        if !page.has_more() {
            break;
        }
        
        offset = page.next_offset();
    }
    
    Ok(all_books)
}
```

#### 3.2.10 Client Cargo.toml

```toml
[package]
name = "api_client"
version.workspace = true
edition.workspace = true
description = "Rust client SDK for the e-reader API"

[features]
default = ["rustls"]
rustls = ["reqwest/rustls-tls"]
native-tls = ["reqwest/native-tls"]
offline = []  # Enable offline support

[dependencies]
# HTTP
reqwest = { version = "0.11", default-features = false, features = ["json", "multipart"] }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
serde_urlencoded = "0.7"

# Async
tokio = { workspace = true, features = ["fs"] }
async-trait = "0.1"

# Types
uuid = { workspace = true }
chrono = { workspace = true }

# Error handling
thiserror = { workspace = true }

[dev-dependencies]
tokio-test = { workspace = true }
wiremock = "0.5"
```

---

### 3.3 `db_layer` Crate

Database access layer with SQLx.

```rust
// crates/db_layer/src/lib.rs
pub mod models;
pub mod queries;
pub mod pool;

pub use pool::DbPool;
```

#### 3.2.1 Connection Pool

```rust
// crates/db_layer/src/pool.rs
use common::config::DatabaseConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub type DbPool = PgPool;

pub async fn create_pool(config: &DatabaseConfig) -> sqlx::Result<DbPool> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .connect(&config.url)
        .await
}

pub async fn run_migrations(pool: &DbPool) -> sqlx::Result<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}
```

#### 3.2.2 Models

```rust
// crates/db_layer/src/models/book.rs
use chrono::{DateTime, Utc};
use common::types::{BookFormat, BookId, ContentHash, UserId};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Book {
    pub id: BookId,
    pub user_id: UserId,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct FileAsset {
    pub id: common::types::FileAssetId,
    pub book_id: BookId,
    pub format: BookFormat,
    pub file_size: i64,
    pub content_hash: ContentHash,
    pub storage_path: String,
    pub original_filename: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Cover {
    pub id: uuid::Uuid,
    pub book_id: BookId,
    pub size_variant: String, // "small", "medium", "large"
    pub width: i32,
    pub height: i32,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
}

// crates/db_layer/src/models/reading_state.rs
use chrono::{DateTime, Utc};
use common::types::{BookId, DeviceId, ReadingLocation, UserId};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ReadingState {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub book_id: BookId,
    pub device_id: DeviceId,
    pub location: sqlx::types::Json<ReadingLocation>,
    pub updated_at: DateTime<Utc>,
}

// crates/db_layer/src/models/annotation.rs
use chrono::{DateTime, Utc};
use common::types::{AnnotationId, BookId, UserId};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "annotation_type", rename_all = "lowercase")]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Annotation {
    pub id: AnnotationId,
    pub user_id: UserId,
    pub book_id: BookId,
    pub annotation_type: AnnotationType,
    pub location_start: String,
    pub location_end: Option<String>,
    pub content: Option<String>,
    pub style: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>, // soft delete for sync
}

// crates/db_layer/src/models/device.rs
use chrono::{DateTime, Utc};
use common::types::{DeviceId, UserId};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Device {
    pub id: DeviceId,
    pub user_id: UserId,
    pub name: String,
    pub device_type: String,
    pub public_key: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// crates/db_layer/src/models/collection.rs
use chrono::{DateTime, Utc};
use common::types::{BookId, CollectionId, UserId};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Collection {
    pub id: CollectionId,
    pub user_id: UserId,
    pub name: String,
    pub description: Option<String>,
    pub collection_type: String, // "shelf", "tag", "series"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// crates/db_layer/src/models/mod.rs
pub mod annotation;
pub mod book;
pub mod collection;
pub mod device;
pub mod reading_state;

pub use annotation::*;
pub use book::*;
pub use collection::*;
pub use device::*;
pub use reading_state::*;
```

#### 3.2.3 Queries

```rust
// crates/db_layer/src/queries/books.rs
use crate::models::{Book, Cover, FileAsset};
use crate::DbPool;
use common::error::Result;
use common::types::{BookId, ContentHash, Pagination, Paginated, UserId};

pub struct BookQueries;

impl BookQueries {
    pub async fn list_for_user(
        pool: &DbPool,
        user_id: &UserId,
        pagination: &Pagination,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
    ) -> Result<Paginated<Book>> {
        let sort_column = match sort_by.unwrap_or("created_at") {
            "title" => "title",
            "author" => "authors[1]",
            "updated_at" => "updated_at",
            _ => "created_at",
        };
        let order = if sort_order == Some("asc") { "ASC" } else { "DESC" };
        
        // Note: In production, use a macro or query builder for dynamic ordering
        let items = sqlx::query_as!(
            Book,
            r#"
            SELECT id, user_id, title, authors, description, language, 
                   publisher, published_date, isbn, series_name, series_index,
                   tags, created_at, updated_at
            FROM books
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id.0,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(pool)
        .await?;
        
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM books WHERE user_id = $1",
            user_id.0
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
        
        Ok(Paginated {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }
    
    pub async fn get_by_id(pool: &DbPool, id: &BookId, user_id: &UserId) -> Result<Option<Book>> {
        let book = sqlx::query_as!(
            Book,
            r#"
            SELECT id, user_id, title, authors, description, language,
                   publisher, published_date, isbn, series_name, series_index,
                   tags, created_at, updated_at
            FROM books
            WHERE id = $1 AND user_id = $2
            "#,
            id.0,
            user_id.0
        )
        .fetch_optional(pool)
        .await?;
        
        Ok(book)
    }
    
    pub async fn find_by_hash(
        pool: &DbPool,
        user_id: &UserId,
        hash: &ContentHash,
    ) -> Result<Option<Book>> {
        let book = sqlx::query_as!(
            Book,
            r#"
            SELECT b.id, b.user_id, b.title, b.authors, b.description, b.language,
                   b.publisher, b.published_date, b.isbn, b.series_name, b.series_index,
                   b.tags, b.created_at, b.updated_at
            FROM books b
            INNER JOIN file_assets fa ON fa.book_id = b.id
            WHERE b.user_id = $1 AND fa.content_hash = $2
            "#,
            user_id.0,
            hash.0
        )
        .fetch_optional(pool)
        .await?;
        
        Ok(book)
    }
    
    pub async fn create(pool: &DbPool, book: &CreateBook) -> Result<Book> {
        let book = sqlx::query_as!(
            Book,
            r#"
            INSERT INTO books (id, user_id, title, authors, description, language,
                              publisher, published_date, isbn, series_name, series_index, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, user_id, title, authors, description, language,
                      publisher, published_date, isbn, series_name, series_index,
                      tags, created_at, updated_at
            "#,
            book.id.0,
            book.user_id.0,
            book.title,
            &book.authors,
            book.description,
            book.language,
            book.publisher,
            book.published_date,
            book.isbn,
            book.series_name,
            book.series_index,
            &book.tags
        )
        .fetch_one(pool)
        .await?;
        
        Ok(book)
    }
    
    pub async fn update_metadata(
        pool: &DbPool,
        id: &BookId,
        user_id: &UserId,
        update: &UpdateBookMetadata,
    ) -> Result<Book> {
        let book = sqlx::query_as!(
            Book,
            r#"
            UPDATE books SET
                title = COALESCE($3, title),
                authors = COALESCE($4, authors),
                description = COALESCE($5, description),
                series_name = COALESCE($6, series_name),
                series_index = COALESCE($7, series_index),
                tags = COALESCE($8, tags),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            RETURNING id, user_id, title, authors, description, language,
                      publisher, published_date, isbn, series_name, series_index,
                      tags, created_at, updated_at
            "#,
            id.0,
            user_id.0,
            update.title,
            update.authors.as_deref(),
            update.description,
            update.series_name,
            update.series_index,
            update.tags.as_deref()
        )
        .fetch_one(pool)
        .await?;
        
        Ok(book)
    }
    
    pub async fn delete(pool: &DbPool, id: &BookId, user_id: &UserId) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM books WHERE id = $1 AND user_id = $2",
            id.0,
            user_id.0
        )
        .execute(pool)
        .await?;
        
        Ok(result.rows_affected() > 0)
    }
    
    pub async fn search(
        pool: &DbPool,
        user_id: &UserId,
        query: &str,
        pagination: &Pagination,
    ) -> Result<Paginated<Book>> {
        // Basic search using ILIKE - can be upgraded to full-text search later
        let pattern = format!("%{}%", query);
        
        let items = sqlx::query_as!(
            Book,
            r#"
            SELECT id, user_id, title, authors, description, language,
                   publisher, published_date, isbn, series_name, series_index,
                   tags, created_at, updated_at
            FROM books
            WHERE user_id = $1 
              AND (title ILIKE $2 OR $2 ILIKE ANY(authors) OR series_name ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id.0,
            pattern,
            pagination.limit,
            pagination.offset
        )
        .fetch_all(pool)
        .await?;
        
        let total = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM books
            WHERE user_id = $1 
              AND (title ILIKE $2 OR $2 ILIKE ANY(authors) OR series_name ILIKE $2)
            "#,
            user_id.0,
            pattern
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);
        
        Ok(Paginated {
            items,
            total,
            limit: pagination.limit,
            offset: pagination.offset,
        })
    }
}

#[derive(Debug)]
pub struct CreateBook {
    pub id: BookId,
    pub user_id: UserId,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
}

#[derive(Debug, Default)]
pub struct UpdateBookMetadata {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Option<Vec<String>>,
}

// crates/db_layer/src/queries/mod.rs
pub mod annotations;
pub mod books;
pub mod collections;
pub mod devices;
pub mod file_assets;
pub mod reading_states;
pub mod tasks;

pub use books::BookQueries;
// ... other exports
```

---

### 3.4 `storage_layer` Crate

File storage abstraction.

```rust
// crates/storage_layer/src/lib.rs
pub mod local;
pub mod traits;

pub use local::LocalStorage;
pub use traits::Storage;
```

```rust
// crates/storage_layer/src/traits.rs
use async_trait::async_trait;
use common::error::Result;
use common::types::ContentHash;
use std::path::Path;
use tokio::io::AsyncRead;

/// Storage abstraction trait
#[async_trait]
pub trait Storage: Send + Sync {
    /// Store file from reader, returns storage path
    async fn store<R: AsyncRead + Send + Unpin>(
        &self,
        reader: R,
        filename: &str,
        content_hash: &ContentHash,
    ) -> Result<String>;
    
    /// Store file from bytes
    async fn store_bytes(
        &self,
        data: &[u8],
        filename: &str,
        content_hash: &ContentHash,
    ) -> Result<String>;
    
    /// Get file as bytes
    async fn get(&self, path: &str) -> Result<Vec<u8>>;
    
    /// Get file size
    async fn size(&self, path: &str) -> Result<u64>;
    
    /// Check if file exists
    async fn exists(&self, path: &str) -> Result<bool>;
    
    /// Delete file
    async fn delete(&self, path: &str) -> Result<()>;
    
    /// Get absolute filesystem path (for serving)
    fn absolute_path(&self, path: &str) -> std::path::PathBuf;
}

/// Cover storage with size variants
#[async_trait]
pub trait CoverStorage: Send + Sync {
    async fn store_cover(
        &self,
        image_data: &[u8],
        book_id: &str,
    ) -> Result<CoverPaths>;
    
    async fn get_cover(&self, path: &str) -> Result<Vec<u8>>;
    
    async fn delete_covers(&self, book_id: &str) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct CoverPaths {
    pub small: String,   // ~80px
    pub medium: String,  // ~200px
    pub large: String,   // ~400px
}
```

```rust
// crates/storage_layer/src/local.rs
use crate::traits::{CoverPaths, CoverStorage, Storage};
use async_trait::async_trait;
use common::error::{Error, Result};
use common::types::ContentHash;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

pub struct LocalStorage {
    base_path: PathBuf,
    covers_path: PathBuf,
}

impl LocalStorage {
    pub async fn new(base_path: impl AsRef<Path>, covers_path: impl AsRef<Path>) -> Result<Self> {
        let base = base_path.as_ref().to_path_buf();
        let covers = covers_path.as_ref().to_path_buf();
        
        fs::create_dir_all(&base).await.map_err(|e| Error::Storage(e.to_string()))?;
        fs::create_dir_all(&covers).await.map_err(|e| Error::Storage(e.to_string()))?;
        
        Ok(Self {
            base_path: base,
            covers_path: covers,
        })
    }
    
    fn content_path(&self, hash: &ContentHash, extension: &str) -> PathBuf {
        // Use first 2 chars of hash as subdirectory for distribution
        let subdir = &hash.0[..2];
        self.base_path.join(subdir).join(format!("{}.{}", hash.0, extension))
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn store<R: AsyncRead + Send + Unpin>(
        &self,
        mut reader: R,
        filename: &str,
        content_hash: &ContentHash,
    ) -> Result<String> {
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        
        let path = self.content_path(content_hash, extension);
        
        // Create parent directory
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| Error::Storage(e.to_string()))?;
        }
        
        // Write file
        let mut file = fs::File::create(&path).await.map_err(|e| Error::Storage(e.to_string()))?;
        let mut buffer = vec![0u8; 64 * 1024];
        
        loop {
            let n = reader.read(&mut buffer).await.map_err(|e| Error::Storage(e.to_string()))?;
            if n == 0 { break; }
            file.write_all(&buffer[..n]).await.map_err(|e| Error::Storage(e.to_string()))?;
        }
        
        // Return relative path
        Ok(path.strip_prefix(&self.base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| path.to_string_lossy().to_string()))
    }
    
    async fn store_bytes(
        &self,
        data: &[u8],
        filename: &str,
        content_hash: &ContentHash,
    ) -> Result<String> {
        let cursor = std::io::Cursor::new(data.to_vec());
        self.store(cursor, filename, content_hash).await
    }
    
    async fn get(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.base_path.join(path);
        fs::read(&full_path).await.map_err(|e| Error::Storage(e.to_string()))
    }
    
    async fn size(&self, path: &str) -> Result<u64> {
        let full_path = self.base_path.join(path);
        let metadata = fs::metadata(&full_path).await.map_err(|e| Error::Storage(e.to_string()))?;
        Ok(metadata.len())
    }
    
    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = self.base_path.join(path);
        Ok(full_path.exists())
    }
    
    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.base_path.join(path);
        fs::remove_file(&full_path).await.map_err(|e| Error::Storage(e.to_string()))
    }
    
    fn absolute_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }
}

#[async_trait]
impl CoverStorage for LocalStorage {
    async fn store_cover(&self, image_data: &[u8], book_id: &str) -> Result<CoverPaths> {
        use image::GenericImageView;
        
        let img = image::load_from_memory(image_data)
            .map_err(|e| Error::Storage(format!("Invalid image: {}", e)))?;
        
        let sizes = [
            ("small", 80u32),
            ("medium", 200u32),
            ("large", 400u32),
        ];
        
        let mut paths = CoverPaths {
            small: String::new(),
            medium: String::new(),
            large: String::new(),
        };
        
        for (name, width) in sizes {
            let resized = img.resize(width, width * 3 / 2, image::imageops::FilterType::Lanczos3);
            let path = self.covers_path.join(format!("{}_{}.jpg", book_id, name));
            
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).await.map_err(|e| Error::Storage(e.to_string()))?;
            }
            
            resized.save(&path).map_err(|e| Error::Storage(e.to_string()))?;
            
            let relative = path.strip_prefix(&self.covers_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            
            match name {
                "small" => paths.small = relative,
                "medium" => paths.medium = relative,
                "large" => paths.large = relative,
                _ => {}
            }
        }
        
        Ok(paths)
    }
    
    async fn get_cover(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = self.covers_path.join(path);
        fs::read(&full_path).await.map_err(|e| Error::Storage(e.to_string()))
    }
    
    async fn delete_covers(&self, book_id: &str) -> Result<()> {
        for size in ["small", "medium", "large"] {
            let path = self.covers_path.join(format!("{}_{}.jpg", book_id, size));
            if path.exists() {
                let _ = fs::remove_file(&path).await;
            }
        }
        Ok(())
    }
}
```

---

### 3.5 `indexer` Crate

Metadata extraction and cover generation.

```rust
// crates/indexer/src/lib.rs
pub mod cover;
pub mod epub;
pub mod pdf;
pub mod traits;

pub use traits::{BookMetadata, FormatHandler};

use common::error::Result;
use common::types::BookFormat;

pub fn handler_for_format(format: BookFormat) -> Box<dyn FormatHandler> {
    match format {
        BookFormat::Epub => Box::new(epub::EpubHandler),
        BookFormat::Pdf => Box::new(pdf::PdfHandler),
        BookFormat::Cbz => Box::new(pdf::PdfHandler), // placeholder
        BookFormat::Mobi => Box::new(pdf::PdfHandler), // placeholder
    }
}
```

```rust
// crates/indexer/src/traits.rs
use common::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub isbn: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub cover_data: Option<Vec<u8>>,
    pub page_count: Option<u32>,
}

pub trait FormatHandler: Send + Sync {
    /// Extract metadata from file bytes
    fn extract_metadata(&self, data: &[u8]) -> Result<BookMetadata>;
    
    /// Extract cover image if present
    fn extract_cover(&self, data: &[u8]) -> Result<Option<Vec<u8>>>;
    
    /// Calculate reading locations/page map
    fn calculate_locations(&self, data: &[u8]) -> Result<Vec<String>>;
}
```

```rust
// crates/indexer/src/epub.rs
use crate::traits::{BookMetadata, FormatHandler};
use common::error::{Error, Result};
use epub::doc::EpubDoc;
use std::io::Cursor;

pub struct EpubHandler;

impl FormatHandler for EpubHandler {
    fn extract_metadata(&self, data: &[u8]) -> Result<BookMetadata> {
        let cursor = Cursor::new(data);
        let doc = EpubDoc::from_reader(cursor)
            .map_err(|e| Error::Internal(format!("Failed to parse EPUB: {}", e)))?;
        
        let mut metadata = BookMetadata::default();
        
        metadata.title = doc.mdata("title");
        metadata.authors = doc.mdata("creator")
            .map(|a| vec![a])
            .unwrap_or_default();
        metadata.description = doc.mdata("description");
        metadata.language = doc.mdata("language");
        metadata.publisher = doc.mdata("publisher");
        metadata.isbn = doc.mdata("identifier");
        
        // Try to extract cover
        if let Ok(Some(cover)) = self.extract_cover(data) {
            metadata.cover_data = Some(cover);
        }
        
        Ok(metadata)
    }
    
    fn extract_cover(&self, data: &[u8]) -> Result<Option<Vec<u8>>> {
        let cursor = Cursor::new(data);
        let mut doc = EpubDoc::from_reader(cursor)
            .map_err(|e| Error::Internal(format!("Failed to parse EPUB: {}", e)))?;
        
        if let Some(cover_data) = doc.get_cover() {
            return Ok(Some(cover_data));
        }
        
        Ok(None)
    }
    
    fn calculate_locations(&self, data: &[u8]) -> Result<Vec<String>> {
        // Return spine item IDs as locations for now
        // A real implementation would calculate CFIs
        let cursor = Cursor::new(data);
        let doc = EpubDoc::from_reader(cursor)
            .map_err(|e| Error::Internal(format!("Failed to parse EPUB: {}", e)))?;
        
        Ok(doc.spine.iter().map(|s| s.clone()).collect())
    }
}

// crates/indexer/src/pdf.rs
use crate::traits::{BookMetadata, FormatHandler};
use common::error::{Error, Result};

pub struct PdfHandler;

impl FormatHandler for PdfHandler {
    fn extract_metadata(&self, data: &[u8]) -> Result<BookMetadata> {
        let doc = lopdf::Document::load_mem(data)
            .map_err(|e| Error::Internal(format!("Failed to parse PDF: {}", e)))?;
        
        let mut metadata = BookMetadata::default();
        
        if let Ok(info) = doc.trailer.get(b"Info") {
            if let Ok(info_ref) = info.as_reference() {
                if let Ok(info_dict) = doc.get_dictionary(info_ref) {
                    metadata.title = info_dict.get(b"Title")
                        .and_then(|v| v.as_str().ok())
                        .map(|s| s.to_string());
                    metadata.authors = info_dict.get(b"Author")
                        .and_then(|v| v.as_str().ok())
                        .map(|s| vec![s.to_string()])
                        .unwrap_or_default();
                }
            }
        }
        
        metadata.page_count = Some(doc.get_pages().len() as u32);
        
        Ok(metadata)
    }
    
    fn extract_cover(&self, _data: &[u8]) -> Result<Option<Vec<u8>>> {
        // PDF cover extraction is complex - render first page
        // For v1, skip this
        Ok(None)
    }
    
    fn calculate_locations(&self, data: &[u8]) -> Result<Vec<String>> {
        let doc = lopdf::Document::load_mem(data)
            .map_err(|e| Error::Internal(format!("Failed to parse PDF: {}", e)))?;
        
        let page_count = doc.get_pages().len();
        Ok((1..=page_count).map(|p| format!("page:{}", p)).collect())
    }
}
```

---

### 3.6 `sync_engine` Crate

Synchronization logic and conflict resolution.

```rust
// crates/sync_engine/src/lib.rs
pub mod batch;
pub mod conflicts;
pub mod merge;

use chrono::{DateTime, Utc};
use common::types::{BookId, DeviceId};
use serde::{Deserialize, Serialize};

/// Sync request from a device
#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub device_id: DeviceId,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub reading_states: Vec<ReadingStateSync>,
    pub annotations: Vec<AnnotationSync>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingStateSync {
    pub book_id: BookId,
    pub location: common::types::ReadingLocation,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnnotationSync {
    pub id: Option<common::types::AnnotationId>,
    pub book_id: BookId,
    pub annotation_type: db_layer::models::AnnotationType,
    pub location_start: String,
    pub location_end: Option<String>,
    pub content: Option<String>,
    pub style: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}

/// Sync response to device
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub server_time: DateTime<Utc>,
    pub reading_states: Vec<ReadingStateSync>,
    pub annotations: Vec<AnnotationSync>,
    pub conflicts: Vec<SyncConflict>,
}

#[derive(Debug, Serialize)]
pub struct SyncConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub local_updated_at: DateTime<Utc>,
    pub server_updated_at: DateTime<Utc>,
    pub resolution: ConflictResolution,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    Merged,
}
```

```rust
// crates/sync_engine/src/merge.rs
use super::*;
use common::error::Result;
use common::types::UserId;
use db_layer::DbPool;

pub struct SyncMerger<'a> {
    pool: &'a DbPool,
    user_id: UserId,
}

impl<'a> SyncMerger<'a> {
    pub fn new(pool: &'a DbPool, user_id: UserId) -> Self {
        Self { pool, user_id }
    }
    
    pub async fn process_sync(&self, request: SyncRequest) -> Result<SyncResponse> {
        let server_time = Utc::now();
        let mut conflicts = Vec::new();
        
        // Process reading states
        let reading_states = self.merge_reading_states(
            &request.device_id,
            &request.reading_states,
            request.last_sync_at,
            &mut conflicts,
        ).await?;
        
        // Process annotations
        let annotations = self.merge_annotations(
            &request.annotations,
            request.last_sync_at,
            &mut conflicts,
        ).await?;
        
        // Update device last_sync_at
        self.update_device_sync_time(&request.device_id, server_time).await?;
        
        Ok(SyncResponse {
            server_time,
            reading_states,
            annotations,
            conflicts,
        })
    }
    
    async fn merge_reading_states(
        &self,
        device_id: &DeviceId,
        incoming: &[ReadingStateSync],
        last_sync: Option<DateTime<Utc>>,
        conflicts: &mut Vec<SyncConflict>,
    ) -> Result<Vec<ReadingStateSync>> {
        // Strategy: Last-write-wins per book
        // A more sophisticated approach would track per-device progress
        
        for state in incoming {
            // Get current server state for this book
            let current = db_layer::queries::reading_states::get_for_book(
                self.pool,
                &self.user_id,
                &state.book_id,
            ).await?;
            
            match current {
                Some(server_state) if server_state.updated_at > state.updated_at => {
                    // Server wins - record conflict
                    conflicts.push(SyncConflict {
                        entity_type: "reading_state".into(),
                        entity_id: state.book_id.0.to_string(),
                        local_updated_at: state.updated_at,
                        server_updated_at: server_state.updated_at,
                        resolution: ConflictResolution::ServerWins,
                    });
                }
                _ => {
                    // Client wins - update server
                    db_layer::queries::reading_states::upsert(
                        self.pool,
                        &self.user_id,
                        &state.book_id,
                        device_id,
                        &state.location,
                    ).await?;
                }
            }
        }
        
        // Return states updated since last sync
        let updated = db_layer::queries::reading_states::get_updated_since(
            self.pool,
            &self.user_id,
            last_sync,
        ).await?;
        
        Ok(updated.into_iter().map(|s| ReadingStateSync {
            book_id: s.book_id,
            location: s.location.0,
            updated_at: s.updated_at,
        }).collect())
    }
    
    async fn merge_annotations(
        &self,
        incoming: &[AnnotationSync],
        last_sync: Option<DateTime<Utc>>,
        conflicts: &mut Vec<SyncConflict>,
    ) -> Result<Vec<AnnotationSync>> {
        // Similar last-write-wins strategy
        // Annotations can be soft-deleted for sync
        
        for ann in incoming {
            if let Some(id) = &ann.id {
                let current = db_layer::queries::annotations::get_by_id(
                    self.pool,
                    id,
                    &self.user_id,
                ).await?;
                
                if let Some(server_ann) = current {
                    if server_ann.updated_at > ann.updated_at {
                        conflicts.push(SyncConflict {
                            entity_type: "annotation".into(),
                            entity_id: id.0.to_string(),
                            local_updated_at: ann.updated_at,
                            server_updated_at: server_ann.updated_at,
                            resolution: ConflictResolution::ServerWins,
                        });
                        continue;
                    }
                }
            }
            
            // Upsert annotation
            db_layer::queries::annotations::upsert(
                self.pool,
                &self.user_id,
                ann,
            ).await?;
        }
        
        // Return annotations updated since last sync
        let updated = db_layer::queries::annotations::get_updated_since(
            self.pool,
            &self.user_id,
            last_sync,
        ).await?;
        
        Ok(updated.into_iter().map(|a| AnnotationSync {
            id: Some(a.id),
            book_id: a.book_id,
            annotation_type: a.annotation_type,
            location_start: a.location_start,
            location_end: a.location_end,
            content: a.content,
            style: a.style,
            updated_at: a.updated_at,
            deleted: a.deleted_at.is_some(),
        }).collect())
    }
    
    async fn update_device_sync_time(
        &self,
        device_id: &DeviceId,
        time: DateTime<Utc>,
    ) -> Result<()> {
        db_layer::queries::devices::update_last_sync(
            self.pool,
            device_id,
            &self.user_id,
            time,
        ).await
    }
}
```

---

### 3.7 `api_server` Crate

The main HTTP API server.

```rust
// crates/api_server/src/main.rs
use api_server::{create_app, AppState};
use common::config::AppConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    dotenvy::dotenv().ok();
    
    let config = AppConfig::load()?;
    
    tracing::info!("Starting e-reader API server on {}:{}", config.server.host, config.server.port);
    
    // Create database pool
    let pool = db_layer::pool::create_pool(&config.database).await?;
    db_layer::pool::run_migrations(&pool).await?;
    
    // Create storage
    let storage = storage_layer::LocalStorage::new(
        &config.storage.base_path,
        &config.storage.covers_path,
    ).await?;
    
    // Create app state
    let state = AppState::new(config.clone(), pool, storage);
    
    // Build application
    let app = create_app(state);
    
    // Run server
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.server.host,
        config.server.port
    )).await?;
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

```rust
// crates/api_server/src/lib.rs
pub mod error;
pub mod extractors;
pub mod middleware;
pub mod routes;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use common::config::AppConfig;
use db_layer::DbPool;
use std::sync::Arc;
use storage_layer::LocalStorage;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub pool: DbPool,
    pub storage: Arc<LocalStorage>,
    pub clerk_client: Arc<middleware::clerk::ClerkClient>,
}

impl AppState {
    pub fn new(config: AppConfig, pool: DbPool, storage: LocalStorage) -> Self {
        let clerk_client = middleware::clerk::ClerkClient::new(&config.clerk);
        Self {
            config,
            pool,
            storage: Arc::new(storage),
            clerk_client: Arc::new(clerk_client),
        }
    }
}

pub fn create_app(state: AppState) -> Router {
    let api_routes = Router::new()
        // Health
        .route("/health", get(routes::health::health_check))
        .route("/health/ready", get(routes::health::readiness_check))
        
        // Auth / Devices
        .route("/auth/device", post(routes::auth::register_device))
        .route("/auth/webhook", post(routes::auth::clerk_webhook))
        
        // Library
        .route("/books", get(routes::library::list_books))
        .route("/books", post(routes::library::create_book))
        .route("/books/search", get(routes::library::search_books))
        .route("/books/:id", get(routes::library::get_book))
        .route("/books/:id", put(routes::library::update_book))
        .route("/books/:id", delete(routes::library::delete_book))
        
        // Assets
        .route("/books/:id/upload", post(routes::assets::upload_file))
        .route("/books/:id/download", get(routes::assets::download_file))
        .route("/books/:id/download/:format", get(routes::assets::download_file_format))
        
        // Covers
        .route("/books/:id/cover", get(routes::covers::get_cover))
        .route("/books/:id/cover/:size", get(routes::covers::get_cover_size))
        .route("/books/:id/cover", post(routes::covers::upload_cover))
        
        // Collections
        .route("/collections", get(routes::collections::list_collections))
        .route("/collections", post(routes::collections::create_collection))
        .route("/collections/:id", get(routes::collections::get_collection))
        .route("/collections/:id", put(routes::collections::update_collection))
        .route("/collections/:id", delete(routes::collections::delete_collection))
        .route("/collections/:id/books", post(routes::collections::add_book))
        .route("/collections/:id/books/:book_id", delete(routes::collections::remove_book))
        
        // Sync
        .route("/sync", post(routes::sync::sync_batch))
        .route("/sync/reading-state/:book_id", get(routes::sync::get_reading_state))
        .route("/sync/reading-state/:book_id", put(routes::sync::update_reading_state))
        .route("/sync/annotations", get(routes::sync::list_annotations))
        .route("/sync/annotations/:book_id", get(routes::sync::get_book_annotations))
        
        // Admin
        .route("/admin/reindex", post(routes::admin::trigger_reindex))
        .route("/admin/backup", post(routes::admin::create_backup))
        .route("/admin/stats", get(routes::admin::get_stats));
    
    Router::new()
        .nest("/api/v1", api_routes)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()) // Configure properly for production
        .layer(RequestBodyLimitLayer::new(state.config.server.request_body_limit))
        .with_state(state)
}
```

#### 3.6.1 Clerk Authentication Middleware

```rust
// crates/api_server/src/middleware/clerk.rs
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use common::config::ClerkConfig;
use common::error::Error;
use common::types::UserId;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct ClerkClient {
    http_client: Client,
    jwks_url: String,
    jwks_cache: std::sync::Arc<RwLock<Option<JwksCache>>>,
}

#[derive(Debug, Clone)]
struct JwksCache {
    keys: HashMap<String, DecodingKey>,
    fetched_at: std::time::Instant,
}

#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<JwkKey>,
}

#[derive(Debug, Deserialize)]
struct JwkKey {
    kid: String,
    kty: String,
    n: String,
    e: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClerkClaims {
    pub sub: String,        // User ID
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    #[serde(default)]
    pub azp: Option<String>,
}

impl ClerkClient {
    pub fn new(config: &ClerkConfig) -> Self {
        Self {
            http_client: Client::new(),
            jwks_url: config.jwks_url.clone(),
            jwks_cache: std::sync::Arc::new(RwLock::new(None)),
        }
    }
    
    pub async fn verify_token(&self, token: &str) -> Result<ClerkClaims, Error> {
        let header = decode_header(token)
            .map_err(|e| Error::Unauthorized(format!("Invalid token header: {}", e)))?;
        
        let kid = header.kid
            .ok_or_else(|| Error::Unauthorized("Token missing kid".into()))?;
        
        let key = self.get_decoding_key(&kid).await?;
        
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&["your-clerk-frontend-api"]);
        
        let token_data = decode::<ClerkClaims>(token, &key, &validation)
            .map_err(|e| Error::Unauthorized(format!("Token validation failed: {}", e)))?;
        
        Ok(token_data.claims)
    }
    
    async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey, Error> {
        // Check cache first
        {
            let cache = self.jwks_cache.read().unwrap();
            if let Some(ref c) = *cache {
                if c.fetched_at.elapsed() < std::time::Duration::from_secs(3600) {
                    if let Some(key) = c.keys.get(kid) {
                        return Ok(key.clone());
                    }
                }
            }
        }
        
        // Fetch fresh JWKS
        let response: JwksResponse = self.http_client
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| Error::ExternalService(format!("JWKS fetch failed: {}", e)))?
            .json()
            .await
            .map_err(|e| Error::ExternalService(format!("JWKS parse failed: {}", e)))?;
        
        let mut keys = HashMap::new();
        for jwk in response.keys {
            if jwk.kty == "RSA" {
                let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)
                    .map_err(|e| Error::ExternalService(format!("Invalid JWK: {}", e)))?;
                keys.insert(jwk.kid, key);
            }
        }
        
        let key = keys.get(kid)
            .ok_or_else(|| Error::Unauthorized("Key not found".into()))?
            .clone();
        
        // Update cache
        {
            let mut cache = self.jwks_cache.write().unwrap();
            *cache = Some(JwksCache {
                keys,
                fetched_at: std::time::Instant::now(),
            });
        }
        
        Ok(key)
    }
}

/// Authenticated user extracted from JWT
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
    pub claims: ClerkClaims,
}

/// Extract Authorization header and validate JWT
pub async fn auth_middleware(
    State(state): State<crate::AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claims = state.clerk_client
        .verify_token(token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user = AuthenticatedUser {
        user_id: UserId(claims.sub.clone()),
        claims,
    };
    
    request.extensions_mut().insert(user);
    
    Ok(next.run(request).await)
}
```

#### 3.6.2 Route Handlers

```rust
// crates/api_server/src/routes/library.rs
use axum::{
    extract::{Path, Query, State},
    Json,
};
use common::error::{Error, Result};
use common::types::{BookId, Pagination, Paginated};
use db_layer::models::Book;
use db_layer::queries::{BookQueries, CreateBook, UpdateBookMetadata};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ListBooksQuery {
    #[serde(flatten)]
    pub pagination: Pagination,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub tag: Option<String>,
    pub series: Option<String>,
}

pub async fn list_books(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<ListBooksQuery>,
) -> Result<Json<Paginated<BookResponse>>> {
    let books = BookQueries::list_for_user(
        &state.pool,
        &user.user_id,
        &query.pagination,
        query.sort_by.as_deref(),
        query.sort_order.as_deref(),
    ).await?;
    
    Ok(Json(Paginated {
        items: books.items.into_iter().map(BookResponse::from).collect(),
        total: books.total,
        limit: books.limit,
        offset: books.offset,
    }))
}

pub async fn get_book(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<BookDetailResponse>> {
    let book_id = BookId(id);
    
    let book = BookQueries::get_by_id(&state.pool, &book_id, &user.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;
    
    // Get file assets
    let assets = db_layer::queries::file_assets::get_for_book(&state.pool, &book_id).await?;
    
    // Get cover URLs
    let covers = db_layer::queries::covers::get_for_book(&state.pool, &book_id).await?;
    
    Ok(Json(BookDetailResponse {
        book: BookResponse::from(book),
        formats: assets.into_iter().map(|a| FormatInfo {
            format: a.format,
            file_size: a.file_size,
        }).collect(),
        covers: covers.into_iter().map(|c| CoverInfo {
            size: c.size_variant,
            url: format!("/api/v1/books/{}/cover/{}", id, c.size_variant),
        }).collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct CreateBookRequest {
    pub title: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Option<Vec<String>>,
}

pub async fn create_book(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<CreateBookRequest>,
) -> Result<Json<BookResponse>> {
    let book = BookQueries::create(&state.pool, &CreateBook {
        id: BookId::new(),
        user_id: user.user_id,
        title: req.title,
        authors: req.authors.unwrap_or_default(),
        description: req.description,
        language: None,
        publisher: None,
        published_date: None,
        isbn: None,
        series_name: req.series_name,
        series_index: req.series_index,
        tags: req.tags.unwrap_or_default(),
    }).await?;
    
    Ok(Json(BookResponse::from(book)))
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookRequest {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Option<Vec<String>>,
}

pub async fn update_book(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateBookRequest>,
) -> Result<Json<BookResponse>> {
    let book_id = BookId(id);
    
    let book = BookQueries::update_metadata(&state.pool, &book_id, &user.user_id, &UpdateBookMetadata {
        title: req.title,
        authors: req.authors,
        description: req.description,
        series_name: req.series_name,
        series_index: req.series_index,
        tags: req.tags,
    }).await?;
    
    Ok(Json(BookResponse::from(book)))
}

pub async fn delete_book(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let book_id = BookId(id);
    
    // Delete from storage
    let assets = db_layer::queries::file_assets::get_for_book(&state.pool, &book_id).await?;
    for asset in assets {
        let _ = state.storage.delete(&asset.storage_path).await;
    }
    
    // Delete covers
    let _ = state.storage.delete_covers(&id.to_string()).await;
    
    // Delete from database
    BookQueries::delete(&state.pool, &book_id, &user.user_id).await?;
    
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(flatten)]
    pub pagination: Pagination,
}

pub async fn search_books(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Paginated<BookResponse>>> {
    let books = BookQueries::search(
        &state.pool,
        &user.user_id,
        &query.q,
        &query.pagination,
    ).await?;
    
    Ok(Json(Paginated {
        items: books.items.into_iter().map(BookResponse::from).collect(),
        total: books.total,
        limit: books.limit,
        offset: books.offset,
    }))
}

// Response types
#[derive(Debug, Serialize)]
pub struct BookResponse {
    pub id: Uuid,
    pub title: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub series_name: Option<String>,
    pub series_index: Option<f32>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Book> for BookResponse {
    fn from(book: Book) -> Self {
        Self {
            id: book.id.0,
            title: book.title,
            authors: book.authors,
            description: book.description,
            series_name: book.series_name,
            series_index: book.series_index,
            tags: book.tags,
            created_at: book.created_at,
            updated_at: book.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct BookDetailResponse {
    #[serde(flatten)]
    pub book: BookResponse,
    pub formats: Vec<FormatInfo>,
    pub covers: Vec<CoverInfo>,
}

#[derive(Debug, Serialize)]
pub struct FormatInfo {
    pub format: common::types::BookFormat,
    pub file_size: i64,
}

#[derive(Debug, Serialize)]
pub struct CoverInfo {
    pub size: String,
    pub url: String,
}
```

```rust
// crates/api_server/src/routes/assets.rs
use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use common::error::{Error, Result};
use common::types::{BookFormat, BookId, ContentHash, FileAssetId};
use serde::Deserialize;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::AppState;

pub async fn upload_file(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    let book_id = BookId(id);
    
    // Verify book exists and belongs to user
    let _book = db_layer::queries::BookQueries::get_by_id(&state.pool, &book_id, &user.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;
    
    while let Some(field) = multipart.next_field().await
        .map_err(|e| Error::Validation(e.to_string()))?
    {
        let filename = field.file_name()
            .ok_or_else(|| Error::Validation("Missing filename".into()))?
            .to_string();
        
        let extension = std::path::Path::new(&filename)
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::Validation("Unknown file extension".into()))?;
        
        let format = BookFormat::from_extension(extension)
            .ok_or_else(|| Error::Validation(format!("Unsupported format: {}", extension)))?;
        
        // Read file data
        let data = field.bytes().await
            .map_err(|e| Error::Validation(e.to_string()))?;
        
        let content_hash = ContentHash::from_bytes(&data);
        let file_size = data.len() as i64;
        
        // Check for duplicate
        if let Some(_existing) = db_layer::queries::BookQueries::find_by_hash(
            &state.pool,
            &user.user_id,
            &content_hash,
        ).await? {
            return Err(Error::Conflict("File already exists in library".into()));
        }
        
        // Store file
        let storage_path = state.storage.store_bytes(&data, &filename, &content_hash).await?;
        
        // Extract metadata and update book
        let handler = indexer::handler_for_format(format);
        let metadata = handler.extract_metadata(&data)?;
        
        // Create file asset record
        let asset_id = FileAssetId(Uuid::now_v7());
        db_layer::queries::file_assets::create(
            &state.pool,
            &asset_id,
            &book_id,
            format,
            file_size,
            &content_hash,
            &storage_path,
            &filename,
        ).await?;
        
        // Extract and store cover if present
        if let Some(cover_data) = metadata.cover_data {
            let cover_paths = state.storage.store_cover(&cover_data, &id.to_string()).await?;
            db_layer::queries::covers::create(
                &state.pool,
                &book_id,
                &cover_paths,
            ).await?;
        }
        
        // Update book metadata from file if empty
        if metadata.title.is_some() || !metadata.authors.is_empty() {
            // Only update if book metadata is sparse
            let _ = db_layer::queries::BookQueries::update_metadata(
                &state.pool,
                &book_id,
                &user.user_id,
                &db_layer::queries::UpdateBookMetadata {
                    title: metadata.title,
                    authors: if metadata.authors.is_empty() { None } else { Some(metadata.authors) },
                    description: metadata.description,
                    ..Default::default()
                },
            ).await;
        }
        
        return Ok(Json(UploadResponse {
            asset_id: asset_id.0,
            format,
            file_size,
            content_hash: content_hash.0,
        }));
    }
    
    Err(Error::Validation("No file provided".into()))
}

#[derive(Debug, serde::Serialize)]
pub struct UploadResponse {
    pub asset_id: Uuid,
    pub format: BookFormat,
    pub file_size: i64,
    pub content_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub format: Option<String>,
}

pub async fn download_file(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    download_file_impl(state, user, id, None).await
}

pub async fn download_file_format(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path((id, format)): Path<(Uuid, String)>,
) -> Result<Response> {
    let book_format = BookFormat::from_extension(&format)
        .ok_or_else(|| Error::Validation(format!("Unknown format: {}", format)))?;
    download_file_impl(state, user, id, Some(book_format)).await
}

async fn download_file_impl(
    state: AppState,
    user: crate::middleware::clerk::AuthenticatedUser,
    id: Uuid,
    format: Option<BookFormat>,
) -> Result<Response> {
    let book_id = BookId(id);
    
    // Verify ownership
    let _book = db_layer::queries::BookQueries::get_by_id(&state.pool, &book_id, &user.user_id)
        .await?
        .ok_or_else(|| Error::NotFound("Book not found".into()))?;
    
    // Get file asset
    let assets = db_layer::queries::file_assets::get_for_book(&state.pool, &book_id).await?;
    
    let asset = if let Some(fmt) = format {
        assets.into_iter().find(|a| a.format == fmt)
    } else {
        assets.into_iter().next()
    }.ok_or_else(|| Error::NotFound("No file available".into()))?;
    
    // Open file
    let path = state.storage.absolute_path(&asset.storage_path);
    let file = tokio::fs::File::open(&path).await
        .map_err(|e| Error::Storage(e.to_string()))?;
    
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, asset.format.mime_type())
        .header(header::CONTENT_LENGTH, asset.file_size)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", asset.original_filename),
        )
        .body(body)
        .map_err(|e| Error::Internal(e.to_string()))?;
    
    Ok(response)
}
```

```rust
// crates/api_server/src/routes/sync.rs
use axum::{
    extract::{Path, State},
    Json,
};
use common::error::{Error, Result};
use common::types::BookId;
use sync_engine::{SyncRequest, SyncResponse};
use uuid::Uuid;

use crate::extractors::AuthUser;
use crate::AppState;

pub async fn sync_batch(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(request): Json<SyncRequest>,
) -> Result<Json<SyncResponse>> {
    let merger = sync_engine::merge::SyncMerger::new(&state.pool, user.user_id);
    let response = merger.process_sync(request).await?;
    Ok(Json(response))
}

pub async fn get_reading_state(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(book_id): Path<Uuid>,
) -> Result<Json<Option<ReadingStateResponse>>> {
    let state_opt = db_layer::queries::reading_states::get_for_book(
        &state.pool,
        &user.user_id,
        &BookId(book_id),
    ).await?;
    
    Ok(Json(state_opt.map(|s| ReadingStateResponse {
        book_id: s.book_id.0,
        location: s.location.0,
        updated_at: s.updated_at,
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct UpdateReadingStateRequest {
    pub device_id: Uuid,
    pub location: common::types::ReadingLocation,
}

pub async fn update_reading_state(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(book_id): Path<Uuid>,
    Json(req): Json<UpdateReadingStateRequest>,
) -> Result<Json<ReadingStateResponse>> {
    let book_id = BookId(book_id);
    let device_id = common::types::DeviceId(req.device_id);
    
    db_layer::queries::reading_states::upsert(
        &state.pool,
        &user.user_id,
        &book_id,
        &device_id,
        &req.location,
    ).await?;
    
    let state_record = db_layer::queries::reading_states::get_for_book(
        &state.pool,
        &user.user_id,
        &book_id,
    ).await?.ok_or_else(|| Error::Internal("State not saved".into()))?;
    
    Ok(Json(ReadingStateResponse {
        book_id: state_record.book_id.0,
        location: state_record.location.0,
        updated_at: state_record.updated_at,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct ReadingStateResponse {
    pub book_id: Uuid,
    pub location: common::types::ReadingLocation,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_annotations(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<Vec<AnnotationResponse>>> {
    let annotations = db_layer::queries::annotations::list_for_user(&state.pool, &user.user_id).await?;
    Ok(Json(annotations.into_iter().map(AnnotationResponse::from).collect()))
}

pub async fn get_book_annotations(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(book_id): Path<Uuid>,
) -> Result<Json<Vec<AnnotationResponse>>> {
    let annotations = db_layer::queries::annotations::get_for_book(
        &state.pool,
        &user.user_id,
        &BookId(book_id),
    ).await?;
    Ok(Json(annotations.into_iter().map(AnnotationResponse::from).collect()))
}

#[derive(Debug, serde::Serialize)]
pub struct AnnotationResponse {
    pub id: Uuid,
    pub book_id: Uuid,
    pub annotation_type: db_layer::models::AnnotationType,
    pub location_start: String,
    pub location_end: Option<String>,
    pub content: Option<String>,
    pub style: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<db_layer::models::Annotation> for AnnotationResponse {
    fn from(a: db_layer::models::Annotation) -> Self {
        Self {
            id: a.id.0,
            book_id: a.book_id.0,
            annotation_type: a.annotation_type,
            location_start: a.location_start,
            location_end: a.location_end,
            content: a.content,
            style: a.style,
            created_at: a.created_at,
            updated_at: a.updated_at,
        }
    }
}
```

---

## 4. Database Schema

### 4.1 Initial Migration

```sql
-- migrations/20240101000000_initial_schema.sql

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Custom types
CREATE TYPE book_format AS ENUM ('epub', 'pdf', 'cbz', 'mobi');
CREATE TYPE annotation_type AS ENUM ('highlight', 'note', 'bookmark');

-- Users table (synced from Clerk via webhook)
CREATE TABLE users (
    id TEXT PRIMARY KEY,  -- Clerk user ID
    email TEXT,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Devices
CREATE TABLE devices (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    device_type TEXT NOT NULL DEFAULT 'unknown',
    public_key TEXT,
    last_sync_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_devices_user_id ON devices(user_id);

-- Books
CREATE TABLE books (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    authors TEXT[] NOT NULL DEFAULT '{}',
    description TEXT,
    language TEXT,
    publisher TEXT,
    published_date TEXT,
    isbn TEXT,
    series_name TEXT,
    series_index REAL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_books_user_id ON books(user_id);
CREATE INDEX idx_books_title ON books(user_id, title);
CREATE INDEX idx_books_series ON books(user_id, series_name) WHERE series_name IS NOT NULL;
CREATE INDEX idx_books_created_at ON books(user_id, created_at DESC);

-- File assets
CREATE TABLE file_assets (
    id UUID PRIMARY KEY,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    format book_format NOT NULL,
    file_size BIGINT NOT NULL,
    content_hash TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    original_filename TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(book_id, format)
);

CREATE INDEX idx_file_assets_book_id ON file_assets(book_id);
CREATE INDEX idx_file_assets_content_hash ON file_assets(content_hash);

-- Covers
CREATE TABLE covers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    size_variant TEXT NOT NULL,  -- 'small', 'medium', 'large'
    width INT NOT NULL,
    height INT NOT NULL,
    storage_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(book_id, size_variant)
);

CREATE INDEX idx_covers_book_id ON covers(book_id);

-- Collections/Shelves
CREATE TABLE collections (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    collection_type TEXT NOT NULL DEFAULT 'shelf',  -- 'shelf', 'tag', 'series'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name, collection_type)
);

CREATE INDEX idx_collections_user_id ON collections(user_id);

-- Collection membership
CREATE TABLE collection_books (
    collection_id UUID NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sort_order INT,
    PRIMARY KEY (collection_id, book_id)
);

-- Reading state
CREATE TABLE reading_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    location JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, book_id)
);

CREATE INDEX idx_reading_states_user_book ON reading_states(user_id, book_id);
CREATE INDEX idx_reading_states_updated ON reading_states(user_id, updated_at);

-- Annotations
CREATE TABLE annotations (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    book_id UUID NOT NULL REFERENCES books(id) ON DELETE CASCADE,
    annotation_type annotation_type NOT NULL,
    location_start TEXT NOT NULL,
    location_end TEXT,
    content TEXT,
    style TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ  -- Soft delete for sync
);

CREATE INDEX idx_annotations_user_book ON annotations(user_id, book_id);
CREATE INDEX idx_annotations_updated ON annotations(user_id, updated_at);

-- Background tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- 'pending', 'running', 'completed', 'failed'
    priority INT NOT NULL DEFAULT 0,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tasks_status_scheduled ON tasks(status, scheduled_at) WHERE status = 'pending';
CREATE INDEX idx_tasks_type ON tasks(task_type);

-- Updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply trigger to relevant tables
CREATE TRIGGER update_books_updated_at
    BEFORE UPDATE ON books
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_collections_updated_at
    BEFORE UPDATE ON collections
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_annotations_updated_at
    BEFORE UPDATE ON annotations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

---

## 5. API Endpoint Specifications

### 5.1 Endpoint Summary

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| **Health** |
| GET | `/api/v1/health` | Liveness check | No |
| GET | `/api/v1/health/ready` | Readiness check | No |
| **Auth** |
| POST | `/api/v1/auth/device` | Register device | Yes |
| POST | `/api/v1/auth/webhook` | Clerk webhook | Webhook secret |
| **Library** |
| GET | `/api/v1/books` | List books | Yes |
| POST | `/api/v1/books` | Create book | Yes |
| GET | `/api/v1/books/search` | Search books | Yes |
| GET | `/api/v1/books/:id` | Get book details | Yes |
| PUT | `/api/v1/books/:id` | Update book | Yes |
| DELETE | `/api/v1/books/:id` | Delete book | Yes |
| **Assets** |
| POST | `/api/v1/books/:id/upload` | Upload file | Yes |
| GET | `/api/v1/books/:id/download` | Download file | Yes |
| GET | `/api/v1/books/:id/download/:format` | Download specific format | Yes |
| **Covers** |
| GET | `/api/v1/books/:id/cover` | Get cover (medium) | Yes |
| GET | `/api/v1/books/:id/cover/:size` | Get cover by size | Yes |
| POST | `/api/v1/books/:id/cover` | Upload custom cover | Yes |
| **Collections** |
| GET | `/api/v1/collections` | List collections | Yes |
| POST | `/api/v1/collections` | Create collection | Yes |
| GET | `/api/v1/collections/:id` | Get collection | Yes |
| PUT | `/api/v1/collections/:id` | Update collection | Yes |
| DELETE | `/api/v1/collections/:id` | Delete collection | Yes |
| POST | `/api/v1/collections/:id/books` | Add book | Yes |
| DELETE | `/api/v1/collections/:id/books/:book_id` | Remove book | Yes |
| **Sync** |
| POST | `/api/v1/sync` | Batch sync | Yes |
| GET | `/api/v1/sync/reading-state/:book_id` | Get reading state | Yes |
| PUT | `/api/v1/sync/reading-state/:book_id` | Update reading state | Yes |
| GET | `/api/v1/sync/annotations` | List all annotations | Yes |
| GET | `/api/v1/sync/annotations/:book_id` | Get book annotations | Yes |
| **Admin** |
| POST | `/api/v1/admin/reindex` | Trigger reindex | Yes (admin) |
| POST | `/api/v1/admin/backup` | Create backup | Yes (admin) |
| GET | `/api/v1/admin/stats` | Get statistics | Yes (admin) |

### 5.2 Request/Response Examples

#### List Books

```http
GET /api/v1/books?limit=20&offset=0&sort_by=created_at&sort_order=desc
Authorization: Bearer <clerk_jwt>
```

```json
{
  "items": [
    {
      "id": "019234a1-...",
      "title": "Dune",
      "authors": ["Frank Herbert"],
      "description": "A science fiction masterpiece...",
      "series_name": "Dune",
      "series_index": 1.0,
      "tags": ["sci-fi", "classic"],
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 142,
  "limit": 20,
  "offset": 0
}
```

#### Upload File

```http
POST /api/v1/books/019234a1-.../upload
Authorization: Bearer <clerk_jwt>
Content-Type: multipart/form-data

------boundary
Content-Disposition: form-data; name="file"; filename="dune.epub"
Content-Type: application/epub+zip

<binary data>
------boundary--
```

```json
{
  "asset_id": "019234a2-...",
  "format": "epub",
  "file_size": 1234567,
  "content_hash": "a1b2c3d4..."
}
```

#### Batch Sync

```http
POST /api/v1/sync
Authorization: Bearer <clerk_jwt>
```

```json
{
  "device_id": "019234a3-...",
  "last_sync_at": "2024-01-14T10:00:00Z",
  "reading_states": [
    {
      "book_id": "019234a1-...",
      "location": {
        "locator": "epubcfi(/6/4!/4/2/1:0)",
        "progress": 0.45,
        "chapter": "Chapter 5"
      },
      "updated_at": "2024-01-15T12:00:00Z"
    }
  ],
  "annotations": [
    {
      "id": "019234a4-...",
      "book_id": "019234a1-...",
      "annotation_type": "highlight",
      "location_start": "epubcfi(/6/4!/4/2/1:100)",
      "location_end": "epubcfi(/6/4!/4/2/1:200)",
      "content": "Important quote",
      "style": "yellow",
      "updated_at": "2024-01-15T12:05:00Z",
      "deleted": false
    }
  ]
}
```

Response:
```json
{
  "server_time": "2024-01-15T12:10:00Z",
  "reading_states": [
    {
      "book_id": "019234a5-...",
      "location": { "locator": "page:42", "progress": 0.8, "chapter": null },
      "updated_at": "2024-01-15T11:00:00Z"
    }
  ],
  "annotations": [],
  "conflicts": [
    {
      "entity_type": "reading_state",
      "entity_id": "019234a1-...",
      "local_updated_at": "2024-01-15T12:00:00Z",
      "server_updated_at": "2024-01-15T12:05:00Z",
      "resolution": "server_wins"
    }
  ]
}
```

---

## 6. Authentication with Clerk

### 6.1 Flow Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  Client  │───>│  Clerk   │───>│   API    │───>│ Database │  │
│  │ (Device) │    │  Auth    │    │  Server  │    │          │  │
│  └──────────┘    └──────────┘    └──────────┘    └──────────┘  │
│       │              │                │                        │
│       │  1. Login    │                │                        │
│       │─────────────>│                │                        │
│       │              │                │                        │
│       │  2. JWT      │                │                        │
│       │<─────────────│                │                        │
│       │              │                │                        │
│       │  3. API Request + JWT          │                        │
│       │───────────────────────────────>│                        │
│       │              │                │                        │
│       │              │  4. Verify JWT │                        │
│       │              │<───────────────│                        │
│       │              │                │                        │
│       │              │  5. Valid      │                        │
│       │              │───────────────>│                        │
│       │              │                │                        │
│       │              │                │  6. Query              │
│       │              │                │───────────────────────>│
│       │              │                │                        │
│       │  7. Response │                │                        │
│       │<───────────────────────────────│                        │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Clerk Configuration

1. **Create Clerk Application** at dashboard.clerk.com
2. **Configure JWT Template** (optional, for custom claims)
3. **Set up Webhook** for user sync events

### 6.3 Environment Variables

```bash
EREADER__CLERK__PUBLISHABLE_KEY=pk_test_...
EREADER__CLERK__SECRET_KEY=sk_test_...
EREADER__CLERK__JWKS_URL=https://your-app.clerk.accounts.dev/.well-known/jwks.json
EREADER__CLERK__WEBHOOK_SECRET=whsec_...
```

### 6.4 Device Registration

For e-reader devices, implement a pairing flow:

1. Device generates a short pairing code
2. User enters code on web/mobile client (authenticated)
3. Server associates device with user
4. Device receives device-specific token

```rust
// Device pairing endpoint
pub async fn register_device(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(req): Json<RegisterDeviceRequest>,
) -> Result<Json<DeviceResponse>> {
    // Verify pairing code if provided
    // Create device record
    // Return device credentials
}
```

---

## 7. Error Handling Strategy

### 7.1 Error Response Format

All errors follow a consistent JSON structure:

```json
{
  "error": {
    "code": "not_found",
    "message": "Book not found",
    "details": null
  }
}
```

### 7.2 Error Codes

| HTTP Status | Code | Description |
|-------------|------|-------------|
| 400 | `validation_error` | Invalid request data |
| 401 | `unauthorized` | Missing or invalid auth |
| 403 | `forbidden` | Insufficient permissions |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Resource conflict (duplicate) |
| 500 | `internal_error` | Server error |
| 502 | `external_service_error` | Upstream service failure |

### 7.3 Error Logging

All errors are logged with context:

```rust
tracing::error!(
    error = %e,
    user_id = %user.user_id.0,
    request_id = %request_id,
    "Failed to process request"
);
```

---

## 8. Configuration Management

### 8.1 Configuration Files

```yaml
# config/default.yaml
server:
  host: "0.0.0.0"
  port: 8080
  request_body_limit: 104857600  # 100MB

database:
  url: "postgres://localhost/ereader"
  max_connections: 10
  min_connections: 2

storage:
  base_path: "./data/files"
  covers_path: "./data/covers"
  temp_path: "./data/temp"

clerk:
  publishable_key: ""
  secret_key: ""
  jwks_url: ""
  webhook_secret: ""

worker:
  concurrency: 4
  poll_interval_ms: 1000
```

```yaml
# config/local.yaml (gitignored)
database:
  url: "postgres://user:pass@localhost/ereader_dev"

clerk:
  secret_key: "sk_test_..."
```

### 8.2 Environment Override

All config can be overridden via environment variables:

```bash
EREADER__DATABASE__URL=postgres://...
EREADER__SERVER__PORT=3000
```

---

## 9. Docker & Local Development

### 9.1 docker-compose.yml

```yaml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: ereader-postgres
    environment:
      POSTGRES_USER: ereader
      POSTGRES_PASSWORD: ereader_dev
      POSTGRES_DB: ereader
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ereader"]
      interval: 5s
      timeout: 5s
      retries: 5

  api:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: ereader-api
    environment:
      RUST_LOG: info,tower_http=debug
      EREADER__DATABASE__URL: postgres://ereader:ereader_dev@postgres/ereader
      EREADER__SERVER__HOST: "0.0.0.0"
      EREADER__SERVER__PORT: "8080"
    ports:
      - "8080:8080"
    volumes:
      - ./data:/app/data
    depends_on:
      postgres:
        condition: service_healthy

volumes:
  postgres_data:
```

### 9.2 Dockerfile

```dockerfile
# Build stage
FROM rust:1.75-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY crates/common/Cargo.toml crates/common/
COPY crates/db_layer/Cargo.toml crates/db_layer/
COPY crates/storage_layer/Cargo.toml crates/storage_layer/
COPY crates/indexer/Cargo.toml crates/indexer/
COPY crates/sync_engine/Cargo.toml crates/sync_engine/
COPY crates/api_server/Cargo.toml crates/api_server/
COPY crates/worker_daemon/Cargo.toml crates/worker_daemon/
COPY crates/cli_tool/Cargo.toml crates/cli_tool/

# Create dummy source files for dependency caching
RUN mkdir -p crates/common/src crates/db_layer/src crates/storage_layer/src \
    crates/indexer/src crates/sync_engine/src crates/api_server/src \
    crates/worker_daemon/src crates/cli_tool/src && \
    echo "fn main() {}" > crates/api_server/src/main.rs && \
    echo "fn main() {}" > crates/worker_daemon/src/main.rs && \
    echo "fn main() {}" > crates/cli_tool/src/main.rs && \
    touch crates/common/src/lib.rs crates/db_layer/src/lib.rs \
    crates/storage_layer/src/lib.rs crates/indexer/src/lib.rs \
    crates/sync_engine/src/lib.rs crates/api_server/src/lib.rs \
    crates/worker_daemon/src/lib.rs

RUN cargo build --release --bin api_server

# Copy actual source
COPY crates/ crates/
COPY migrations/ migrations/
COPY config/ config/

# Build
RUN touch crates/*/src/*.rs && cargo build --release --bin api_server

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/api_server /app/
COPY --from=builder /app/config /app/config
COPY --from=builder /app/migrations /app/migrations

RUN mkdir -p /app/data/files /app/data/covers /app/data/temp

EXPOSE 8080

CMD ["/app/api_server"]
```

### 9.3 Development Commands

```bash
# Start database only
docker compose up -d postgres

# Run migrations
export DATABASE_URL=postgres://ereader:ereader_dev@localhost/ereader
sqlx migrate run

# Run server locally
cargo run --bin api_server

# Run with hot reload
cargo watch -x 'run --bin api_server'

# Run tests
cargo test

# Check SQLx queries at compile time
cargo sqlx prepare -- --lib
```

---

## 10. Testing Strategy

### 10.1 Test Structure

```
crates/
├── api_server/
│   └── tests/
│       ├── common/mod.rs      # Test utilities
│       ├── library_test.rs    # Library endpoint tests
│       ├── sync_test.rs       # Sync endpoint tests
│       └── auth_test.rs       # Auth tests
├── db_layer/
│   └── tests/
│       └── queries_test.rs    # Query tests
└── sync_engine/
    └── tests/
        └── merge_test.rs      # Merge logic tests
```

### 10.2 Test Utilities

```rust
// crates/api_server/tests/common/mod.rs
use api_server::{create_app, AppState};
use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

pub struct TestApp {
    pub pool: PgPool,
    pub state: AppState,
}

impl TestApp {
    pub async fn new() -> Self {
        // Create test database
        let pool = create_test_pool().await;
        
        // Run migrations
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        // Create test state with mock auth
        let state = create_test_state(pool.clone()).await;
        
        Self { pool, state }
    }
    
    pub async fn request(&self, req: Request<Body>) -> axum::response::Response {
        let app = create_app(self.state.clone());
        app.oneshot(req).await.unwrap()
    }
    
    pub fn auth_header(&self, user_id: &str) -> String {
        // Generate test JWT
        format!("Bearer test_{}", user_id)
    }
}
```

### 10.3 Integration Test Example

```rust
// crates/api_server/tests/library_test.rs
mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::json;

#[tokio::test]
async fn test_create_and_list_books() {
    let app = common::TestApp::new().await;
    
    // Create a book
    let create_req = Request::builder()
        .method("POST")
        .uri("/api/v1/books")
        .header("Authorization", app.auth_header("test_user"))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "title": "Test Book",
            "authors": ["Test Author"]
        }).to_string()))
        .unwrap();
    
    let response = app.request(create_req).await;
    assert_eq!(response.status(), StatusCode::OK);
    
    // List books
    let list_req = Request::builder()
        .method("GET")
        .uri("/api/v1/books")
        .header("Authorization", app.auth_header("test_user"))
        .body(Body::empty())
        .unwrap();
    
    let response = app.request(list_req).await;
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let books: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(books["total"], 1);
    assert_eq!(books["items"][0]["title"], "Test Book");
}
```

---

## 11. Observability

### 11.1 Logging

Using `tracing` with structured logging:

```rust
tracing::info!(
    book_id = %book.id.0,
    format = ?asset.format,
    size_bytes = asset.file_size,
    "Book file uploaded"
);
```

### 11.2 Metrics Endpoint

```rust
// crates/api_server/src/routes/admin.rs
pub async fn get_stats(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> Result<Json<StatsResponse>> {
    let book_count = sqlx::query_scalar!("SELECT COUNT(*) FROM books")
        .fetch_one(&state.pool)
        .await?;
    
    let storage_bytes = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(file_size), 0) FROM file_assets"
    )
    .fetch_one(&state.pool)
    .await?;
    
    Ok(Json(StatsResponse {
        total_books: book_count.unwrap_or(0),
        total_storage_bytes: storage_bytes.unwrap_or(0),
        // ... more stats
    }))
}
```

### 11.3 Health Checks

```rust
// crates/api_server/src/routes/health.rs
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub async fn readiness_check(
    State(state): State<AppState>,
) -> StatusCode {
    // Check database connection
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
```

---

## 12. Migration Strategy

### 12.1 SQLx Migrations

Migrations are stored in `migrations/` and run automatically on startup:

```bash
# Create new migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info
```

### 12.2 Schema Evolution Guidelines

1. **Never delete columns** in production without a deprecation period
2. **Add columns as nullable** first, backfill, then add constraints
3. **Use database views** for backward compatibility when restructuring
4. **Test migrations** on a copy of production data before deploying

### 12.3 Data Migration Pattern

For complex data transformations:

```sql
-- migrations/20240201000000_add_reading_progress_column.sql

-- Add new column
ALTER TABLE reading_states ADD COLUMN progress_percent REAL;

-- Backfill from JSON location data
UPDATE reading_states
SET progress_percent = (location->>'progress')::REAL
WHERE location ? 'progress';

-- Add default for new records
ALTER TABLE reading_states
ALTER COLUMN progress_percent SET DEFAULT 0.0;
```

---

## Appendix A: Full Crate Dependencies

### api_server/Cargo.toml

```toml
[package]
name = "api_server"
version.workspace = true
edition.workspace = true

[dependencies]
common = { path = "../common" }
db_layer = { path = "../db_layer" }
storage_layer = { path = "../storage_layer" }
indexer = { path = "../indexer" }
sync_engine = { path = "../sync_engine" }

tokio.workspace = true
axum.workspace = true
axum-extra.workspace = true
tower.workspace = true
tower-http.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace = true
chrono.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
anyhow.workspace = true
dotenvy.workspace = true
jsonwebtoken.workspace = true
reqwest.workspace = true

tokio-util = { version = "0.7", features = ["io"] }

[dev-dependencies]
tokio-test.workspace = true
```

---

*Document Version: 1.1*  
*Last Updated: 2025-01*
