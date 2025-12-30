//! JWT validation using Clerk JWKS.

use anyhow::anyhow;
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;
use crate::error::ApiError;

/// Clerk JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClerkClaims {
    pub id: String,  // User ID from Clerk
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

/// JWKS response structure
#[derive(Debug, Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}

/// JSON Web Key
#[derive(Debug, Deserialize)]
struct Jwk {
    kid: String,
    n: String,
    e: String,
}

/// JWT validator with JWKS caching
#[derive(Clone)]
pub struct JwtValidator {
    secret_key: Option<String>,
    jwks_url: Option<String>,
    http_client: Client,
    jwks_cache: Arc<RwLock<HashMap<String, DecodingKey>>>,
}

impl JwtValidator {
    /// Create a new JWT validator
    pub fn new(secret_key: Option<String>, jwks_url: Option<String>) -> Self {
        Self {
            secret_key,
            jwks_url,
            http_client: Client::new(),
            jwks_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Validate a JWT token and extract claims
    pub async fn validate(&self, token: &str) -> Result<ClerkClaims, ApiError> {
        // For development tokens (dev_<user_id>), skip validation
        if token.starts_with("dev_") {
            return Ok(ClerkClaims {
                id: token[4..].to_string(),
                image: None,
                username: None,
                last_name: None,
                first_name: None,
                exp: (chrono::Utc::now().timestamp() + 3600) as usize,
                iat: chrono::Utc::now().timestamp() as usize,
            });
        }

        // Strict mode: require both to be set, and validate via JWKS only
        if !(self.secret_key.is_some() && self.jwks_url.is_some()) {
            return Err(ApiError::Internal(anyhow!(
                "Invalid Clerk configuration: both CLERK_SECRET_KEY and CLERK_JWKS_URL must be set together"
            )));
        }

        let jwks_url = self
            .jwks_url
            .as_ref()
            .ok_or_else(|| ApiError::Internal(anyhow!("JWKS URL missing")))?;

        self.validate_with_jwks(token, jwks_url).await
    }

    async fn validate_with_jwks(&self, token: &str, jwks_url: &str) -> Result<ClerkClaims, ApiError> {
        // Get the key ID from the token header
        let header = decode_header(token).map_err(|e| {
            error!("Failed to decode token header: {:?}", e);
            ApiError::Unauthorized("Invalid token header".to_string())
        })?;

        let kid = header
            .kid
            .ok_or_else(|| ApiError::Unauthorized("Token missing key ID".to_string()))?;

        // Check cache first
        let cached_key = {
            let cache = self.jwks_cache.read().await;
            cache.get(&kid).cloned()
        };

        let decoding_key = match cached_key {
            Some(key) => key,
            None => {
                // Fetch JWKS
                let response: JwksResponse = self
                    .http_client
                    .get(jwks_url)
                    .send()
                    .await
                    .map_err(|e| {
                        error!("Failed to fetch JWKS: {:?}", e);
                        ApiError::Internal(anyhow!("Failed to fetch JWKS"))
                    })?
                    .json()
                    .await
                    .map_err(|e| {
                        error!("Failed to parse JWKS response: {:?}", e);
                        ApiError::Internal(anyhow!("Failed to parse JWKS"))
                    })?;

                // Find the matching key
                let jwk = response.keys.iter().find(|k| k.kid == kid).ok_or_else(|| {
                    ApiError::Unauthorized("Key not found in JWKS".to_string())
                })?;

                // Create decoding key from JWK (used for validation below)
                let _decoding_key =
                    DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|e| {
                        error!("Failed to create decoding key: {:?}", e);
                        ApiError::Internal(anyhow!("Failed to create decoding key"))
                    })?;

                // Cache all keys
                let mut cache = self.jwks_cache.write().await;
                for key in response.keys {
                    if let Ok(dk) = DecodingKey::from_rsa_components(&key.n, &key.e) {
                        cache.insert(key.kid, dk);
                    }
                }

                cache.get(&kid).cloned().ok_or_else(|| {
                    ApiError::Unauthorized("Key not found after caching".to_string())
                })?
            }
        };

        // Validate token
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.validate_exp = true;

        let token_data = decode::<ClerkClaims>(token, &decoding_key, &validation).map_err(|e| {
            error!("JWT validation failed: {:?}", e);
            ApiError::Unauthorized("Invalid or expired token".to_string())
        })?;

        Ok(token_data.claims)
    }
}
