//! API client implementation.

use crate::auth::AuthProvider;
use crate::error::{Error, Result};
use crate::models::{
    Book, Collection, CreateBookRequest, CreateCollectionRequest,
    ListBooksQuery, PaginatedResponse, SearchBooksQuery, SyncRequest,
    SyncResponse, UpdateBookRequest, UpdateCollectionRequest,
};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Client configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub max_retries: u32,
    pub retry_delay: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:3000".to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
        }
    }
}

/// Client builder for fluent configuration
pub struct ClientBuilder {
    config: ClientConfig,
    auth: Option<Arc<dyn AuthProvider>>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
            auth: None,
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    pub fn retry_delay(mut self, delay: Duration) -> Self {
        self.config.retry_delay = delay;
        self
    }

    pub fn auth<A: AuthProvider + 'static>(mut self, auth: A) -> Self {
        self.auth = Some(Arc::new(auth));
        self
    }

    pub fn build(self) -> Result<Client> {
        let http_client = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .build()?;

        Ok(Client {
            config: self.config,
            http: http_client,
            auth: self.auth,
        })
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// API client
pub struct Client {
    config: ClientConfig,
    http: reqwest::Client,
    auth: Option<Arc<dyn AuthProvider>>,
}

impl Client {
    /// Create a new client builder
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Make an authenticated GET request
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        self.request(reqwest::Method::GET, path, Option::<()>::None).await
    }

    /// Make an authenticated POST request
    async fn post<T, B>(&self, path: &str, body: Option<B>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        self.request(reqwest::Method::POST, path, body).await
    }

    /// Make an authenticated PUT request
    async fn put<T, B>(&self, path: &str, body: Option<B>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        self.request(reqwest::Method::PUT, path, body).await
    }

    /// Make an authenticated DELETE request
    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = self.http.delete(&url);

        if let Some(auth) = &self.auth {
            let header = auth.get_auth_header().await?;
            if !header.is_empty() {
                request = request.header(AUTHORIZATION, header);
            }
        }

        let response = self.execute_with_retry(request).await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(Error::from_status(status, text))
        }
    }

    /// Internal request method with retries
    async fn request<T, B>(&self, method: reqwest::Method, path: &str, body: Option<B>) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize,
    {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = self.http.request(method, &url);

        if let Some(auth) = &self.auth {
            let header = auth.get_auth_header().await?;
            if !header.is_empty() {
                request = request.header(AUTHORIZATION, header);
            }
        }

        if let Some(b) = body {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .json(&b);
        }

        let response = self.execute_with_retry(request).await?;

        if response.status().is_success() {
            let body = response.json::<T>().await?;
            Ok(body)
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(Error::from_status(status, text))
        }
    }

    /// Execute a request with retry logic
    async fn execute_with_retry(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<reqwest::Response> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                tokio::time::sleep(self.config.retry_delay * attempt).await;
            }

            // Clone the request for retry
            match request.try_clone() {
                Some(req) => {
                    match req.send().await {
                        Ok(response) => return Ok(response),
                        Err(e) => {
                            tracing::warn!(attempt = attempt, error = %e, "Request failed, will retry");
                            last_error = Some(e);
                        }
                    }
                }
                None => {
                    // Can't clone (e.g., streaming body), try once
                    return request.send().await.map_err(Error::from);
                }
            }
        }

        Err(last_error.map(Error::from).unwrap_or_else(|| {
            Error::Server("Max retries exceeded".to_string())
        }))
    }

    // ==================== Book Endpoints ====================

    /// List books with pagination and filtering
    pub async fn list_books(&self, query: ListBooksQuery) -> Result<PaginatedResponse<Book>> {
        let params = serde_urlencoded::to_string(&query).unwrap_or_default();
        let path = if params.is_empty() {
            "/api/v1/books".to_string()
        } else {
            format!("/api/v1/books?{}", params)
        };
        self.get(&path).await
    }

    /// Get a book by ID
    pub async fn get_book(&self, id: Uuid) -> Result<Book> {
        self.get(&format!("/api/v1/books/{}", id)).await
    }

    /// Create a new book
    pub async fn create_book(&self, request: CreateBookRequest) -> Result<Book> {
        self.post("/api/v1/books", Some(request)).await
    }

    /// Update a book
    pub async fn update_book(&self, id: Uuid, request: UpdateBookRequest) -> Result<Book> {
        self.put(&format!("/api/v1/books/{}", id), Some(request)).await
    }

    /// Delete a book
    pub async fn delete_book(&self, id: Uuid) -> Result<()> {
        self.delete(&format!("/api/v1/books/{}", id)).await
    }

    /// Search books
    pub async fn search_books(&self, query: SearchBooksQuery) -> Result<PaginatedResponse<Book>> {
        let params = serde_urlencoded::to_string(&query).unwrap_or_default();
        self.get(&format!("/api/v1/books/search?{}", params)).await
    }

    // ==================== Collection Endpoints ====================

    /// List collections
    pub async fn list_collections(&self) -> Result<Vec<Collection>> {
        self.get("/api/v1/collections").await
    }

    /// Get a collection by ID
    pub async fn get_collection(&self, id: Uuid) -> Result<Collection> {
        self.get(&format!("/api/v1/collections/{}", id)).await
    }

    /// Create a new collection
    pub async fn create_collection(&self, request: CreateCollectionRequest) -> Result<Collection> {
        self.post("/api/v1/collections", Some(request)).await
    }

    /// Update a collection
    pub async fn update_collection(&self, id: Uuid, request: UpdateCollectionRequest) -> Result<Collection> {
        self.put(&format!("/api/v1/collections/{}", id), Some(request)).await
    }

    /// Delete a collection
    pub async fn delete_collection(&self, id: Uuid) -> Result<()> {
        self.delete(&format!("/api/v1/collections/{}", id)).await
    }

    // ==================== Sync Endpoints ====================

    /// Sync reading progress and annotations
    pub async fn sync(&self, request: SyncRequest) -> Result<SyncResponse> {
        self.post("/api/v1/sync", Some(request)).await
    }

    // ==================== Health Endpoints ====================

    /// Check if the API is healthy
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/api/v1/health", self.config.base_url);
        let response = self.http.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    /// Check if the API is ready
    pub async fn readiness_check(&self) -> Result<bool> {
        let url = format!("{}/api/v1/health/ready", self.config.base_url);
        let response = self.http.get(&url).send().await?;
        Ok(response.status().is_success())
    }
}
