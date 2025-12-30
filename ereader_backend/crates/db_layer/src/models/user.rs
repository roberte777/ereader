//! User model.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User record from the database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Data for creating a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

/// Data for updating an existing user
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateUser {
    pub email: Option<String>,
    pub name: Option<String>,
}
