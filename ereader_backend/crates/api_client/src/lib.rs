//! API client SDK for the e-reader backend.
//!
//! This crate provides a Rust client for interacting with the e-reader API.
//!
//! # Example
//!
//! ```no_run
//! use api_client::{Client, TokenAuth};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), api_client::Error> {
//!     let client = Client::builder()
//!         .base_url("http://localhost:3000")
//!         .auth(TokenAuth::new("your-token"))
//!         .build()?;
//!
//!     let books = client.list_books(Default::default()).await?;
//!     println!("Found {} books", books.total);
//!
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod client;
pub mod error;
pub mod models;

pub use auth::{AuthProvider, DeviceAuth, NoAuth, TokenAuth};
pub use client::{Client, ClientBuilder, ClientConfig};
pub use error::{Error, Result};
pub use models::*;
