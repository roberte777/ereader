//! Local filesystem storage implementation.

use crate::traits::{CoverPaths, CoverSize, CoverStorage, Storage};
use async_trait::async_trait;
use common::{ContentHash, Error, Result};
use image::ImageFormat;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

/// Local filesystem storage
pub struct LocalStorage {
    base_path: PathBuf,
    covers_path: PathBuf,
    temp_path: PathBuf,
}

impl LocalStorage {
    /// Create a new local storage instance
    pub async fn new(
        base_path: impl Into<PathBuf>,
        covers_path: impl Into<PathBuf>,
        temp_path: impl Into<PathBuf>,
    ) -> Result<Self> {
        let storage = Self {
            base_path: base_path.into(),
            covers_path: covers_path.into(),
            temp_path: temp_path.into(),
        };

        // Ensure directories exist
        storage.ensure_directories().await?;

        Ok(storage)
    }

    /// Create storage from config
    pub async fn from_config(config: &common::config::StorageConfig) -> Result<Self> {
        Self::new(&config.base_path, &config.covers_path, &config.temp_path).await
    }

    /// Ensure all storage directories exist
    async fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.base_path).await?;
        fs::create_dir_all(&self.covers_path).await?;
        fs::create_dir_all(&self.temp_path).await?;
        Ok(())
    }

    /// Get the storage path for a content hash (content-addressable)
    fn hash_to_path(&self, hash: &ContentHash) -> PathBuf {
        // Use first 2 chars for directory, next 2 for subdirectory
        let prefix1 = hash.prefix(2);
        let prefix2 = &hash.as_str()[2..4];
        self.base_path.join(prefix1).join(prefix2).join(hash.as_str())
    }

    /// Compute the content hash for data
    pub fn compute_hash(data: &[u8]) -> ContentHash {
        let mut hasher = Sha256::new();
        hasher.update(data);
        ContentHash::from_hex(hex::encode(hasher.finalize()))
    }

    /// Get the covers path
    pub fn covers_path(&self) -> &Path {
        &self.covers_path
    }

    /// Get the temp path
    pub fn temp_path(&self) -> &Path {
        &self.temp_path
    }

    /// Write a temp file and return its path
    pub async fn write_temp(&self, data: &[u8]) -> Result<PathBuf> {
        let temp_name = format!("{}.tmp", Uuid::now_v7());
        let temp_path = self.temp_path.join(&temp_name);
        fs::write(&temp_path, data).await?;
        Ok(temp_path)
    }

    /// Delete a temp file
    pub async fn delete_temp(&self, path: &Path) -> Result<()> {
        if path.starts_with(&self.temp_path) {
            fs::remove_file(path).await.ok();
        }
        Ok(())
    }

    /// Check if storage is healthy (directories exist and are accessible)
    pub async fn health_check(&self) -> bool {
        self.base_path.exists() && self.covers_path.exists() && self.temp_path.exists()
    }
}

#[async_trait]
impl Storage for LocalStorage {
    async fn store(&self, content_hash: &ContentHash, data: &[u8]) -> Result<String> {
        let path = self.hash_to_path(content_hash);

        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write the file
        fs::write(&path, data).await?;

        // Return the relative path from base
        let storage_path = path
            .strip_prefix(&self.base_path)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| content_hash.as_str().to_string());

        tracing::debug!(
            hash = %content_hash,
            path = %storage_path,
            size = data.len(),
            "Stored file"
        );

        Ok(storage_path)
    }

    async fn retrieve(&self, storage_path: &str) -> Result<Vec<u8>> {
        let full_path = self.full_path(storage_path);

        fs::read(&full_path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Error::NotFound(format!("File not found: {}", storage_path))
                } else {
                    Error::Storage(e.to_string())
                }
            })
    }

    async fn exists(&self, storage_path: &str) -> Result<bool> {
        let full_path = self.full_path(storage_path);
        Ok(full_path.exists())
    }

    async fn delete(&self, storage_path: &str) -> Result<bool> {
        let full_path = self.full_path(storage_path);

        match fs::remove_file(&full_path).await {
            Ok(_) => {
                tracing::debug!(path = %storage_path, "Deleted file");
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(Error::Storage(e.to_string())),
        }
    }

    fn full_path(&self, storage_path: &str) -> PathBuf {
        self.base_path.join(storage_path)
    }

    fn base_path(&self) -> &str {
        self.base_path.to_str().unwrap_or("")
    }
}

#[async_trait]
impl CoverStorage for LocalStorage {
    async fn store_cover(&self, book_id: Uuid, image_data: &[u8]) -> Result<CoverPaths> {
        // Load the image
        let img = image::load_from_memory(image_data)
            .map_err(|e| Error::Validation(format!("Invalid image: {}", e)))?;

        // Create book cover directory
        let book_cover_dir = self.covers_path.join(book_id.to_string());
        fs::create_dir_all(&book_cover_dir).await?;

        let mut paths = CoverPaths {
            small: String::new(),
            medium: String::new(),
            large: String::new(),
        };

        // Generate each size
        for size in CoverSize::all() {
            let (width, height) = size.dimensions();
            let resized = img.resize(width, height, image::imageops::FilterType::Lanczos3);

            let filename = format!("{}.jpg", size.as_str());
            let path = book_cover_dir.join(&filename);

            // Save as JPEG
            resized
                .save_with_format(&path, ImageFormat::Jpeg)
                .map_err(|e| Error::Storage(format!("Failed to save cover: {}", e)))?;

            let relative_path = format!("{}/{}", book_id, filename);

            match size {
                CoverSize::Small => paths.small = relative_path,
                CoverSize::Medium => paths.medium = relative_path,
                CoverSize::Large => paths.large = relative_path,
            }
        }

        tracing::debug!(book_id = %book_id, "Generated cover images");

        Ok(paths)
    }

    async fn retrieve_cover(&self, book_id: Uuid, size: CoverSize) -> Result<Vec<u8>> {
        let path = self.covers_path.join(self.cover_path(book_id, size));

        fs::read(&path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Error::NotFound(format!("Cover not found for book {}", book_id))
                } else {
                    Error::Storage(e.to_string())
                }
            })
    }

    async fn cover_exists(&self, book_id: Uuid) -> Result<bool> {
        let path = self.covers_path.join(book_id.to_string()).join("medium.jpg");
        Ok(path.exists())
    }

    async fn delete_cover(&self, book_id: Uuid) -> Result<bool> {
        let book_cover_dir = self.covers_path.join(book_id.to_string());

        if !book_cover_dir.exists() {
            return Ok(false);
        }

        fs::remove_dir_all(&book_cover_dir)
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        tracing::debug!(book_id = %book_id, "Deleted cover images");

        Ok(true)
    }

    fn cover_path(&self, book_id: Uuid, size: CoverSize) -> String {
        format!("{}/{}.jpg", book_id, size.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let data = b"hello world";
        let hash = LocalStorage::compute_hash(data);
        // SHA-256 of "hello world" is known
        assert_eq!(hash.as_str().len(), 64);
    }

    #[test]
    fn test_cover_size_dimensions() {
        assert_eq!(CoverSize::Small.dimensions(), (100, 150));
        assert_eq!(CoverSize::Medium.dimensions(), (200, 300));
        assert_eq!(CoverSize::Large.dimensions(), (400, 600));
    }
}
