use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamespaceRequest {
    /// The name of the namespace to create. Must be unique and between 1 and 6 characters.
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNamespaceResponse {
    /// The name of the created namespace.
    pub name: String,
    /// The timestamp when the namespace was created.
    pub created_at: DateTime<Utc>,
    /// Whether this namespace is the default namespace.
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNamespaceResponse {
    /// The name of the namespace.
    pub name: String,
    /// The timestamp when the namespace was created.
    pub created_at: DateTime<Utc>,
    /// Whether this namespace is the default namespace.
    pub is_default: bool,
}
