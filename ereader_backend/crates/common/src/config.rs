//! Application configuration.

use serde::Deserialize;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub storage: StorageConfig,
    pub clerk: ClerkConfig,
    pub worker: WorkerConfig,
}

/// HTTP server configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_body_limit")]
    pub request_body_limit: usize,
}

fn default_host() -> String {
    "0.0.0.0".into()
}

fn default_port() -> u16 {
    8080
}

fn default_body_limit() -> usize {
    100 * 1024 * 1024 // 100MB
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            request_body_limit: default_body_limit(),
        }
    }
}

impl ServerConfig {
    /// Get the socket address string
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    2
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://ereader:ereader_dev@localhost/ereader".into(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
        }
    }
}

/// File storage configuration
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_base_path")]
    pub base_path: String,
    #[serde(default = "default_covers_path")]
    pub covers_path: String,
    #[serde(default = "default_temp_path")]
    pub temp_path: String,
}

fn default_base_path() -> String {
    "/app/data/files".into()
}

fn default_covers_path() -> String {
    "/app/data/covers".into()
}

fn default_temp_path() -> String {
    "/app/data/temp".into()
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            base_path: default_base_path(),
            covers_path: default_covers_path(),
            temp_path: default_temp_path(),
        }
    }
}

/// Clerk authentication configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ClerkConfig {
    pub publishable_key: String,
    pub secret_key: String,
    #[serde(default = "default_jwks_url")]
    pub jwks_url: String,
    pub webhook_secret: Option<String>,
}

fn default_jwks_url() -> String {
    "https://api.clerk.dev/.well-known/jwks.json".into()
}

impl Default for ClerkConfig {
    fn default() -> Self {
        Self {
            publishable_key: String::new(),
            secret_key: String::new(),
            jwks_url: default_jwks_url(),
            webhook_secret: None,
        }
    }
}

/// Background worker configuration
#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    #[serde(default = "default_max_concurrent_tasks")]
    pub max_concurrent_tasks: u32,
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u32,
    #[serde(default = "default_task_timeout_secs")]
    pub task_timeout_secs: u32,
}

fn default_max_concurrent_tasks() -> u32 {
    4
}

fn default_poll_interval_secs() -> u32 {
    5
}

fn default_task_timeout_secs() -> u32 {
    300
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: default_max_concurrent_tasks(),
            poll_interval_secs: default_poll_interval_secs(),
            task_timeout_secs: default_task_timeout_secs(),
        }
    }
}

impl AppConfig {
    /// Load configuration from files and environment.
    ///
    /// Configuration is loaded in the following order (later sources override earlier):
    /// 1. `config/default.toml`
    /// 2. `config/local.toml` (optional, for local development)
    /// 3. Environment variables (prefix: `EREADER__`, separator: `__`)
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default").required(false))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("EREADER").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    /// Load configuration from a specific file path
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(path))
            .add_source(config::Environment::with_prefix("EREADER").separator("__"))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.request_body_limit, 100 * 1024 * 1024);
    }

    #[test]
    fn test_server_addr() {
        let config = ServerConfig {
            host: "127.0.0.1".into(),
            port: 3000,
            request_body_limit: 1024,
        };
        assert_eq!(config.addr(), "127.0.0.1:3000");
    }

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::default();
        assert_eq!(config.base_path, "/app/data/files");
        assert_eq!(config.covers_path, "/app/data/covers");
        assert_eq!(config.temp_path, "/app/data/temp");
    }

    #[test]
    fn test_worker_config_defaults() {
        let config = WorkerConfig::default();
        assert_eq!(config.max_concurrent_tasks, 4);
        assert_eq!(config.poll_interval_secs, 5);
        assert_eq!(config.task_timeout_secs, 300);
    }
}
