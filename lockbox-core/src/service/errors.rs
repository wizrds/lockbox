use thiserror::Error;

use crate::database::errors::DatabaseError;


#[derive(Error, Debug)]
pub enum ApiKeyServiceError {
    #[error("API key not found")]
    KeyNotFound,
    #[error("API key expired")]
    KeyExpired,
    #[error("Invalid API key")]
    InvalidKey,
    #[error("Insufficient scope")]
    InsufficientScope,
    #[error("API key already exists")]
    KeyAlreadyExists,
    #[error("Namespace not found")]
    NamespaceNotFound,
    #[error("Namespace already exists")]
    NamespaceAlreadyExists,
    #[error("Tag not found")]
    TagNotFound,
    #[error("Tag already exists")]
    TagAlreadyExists,
    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

pub type ApiKeyServiceResult<T> = Result<T, ApiKeyServiceError>;
