use anyhow::Error;
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

use lockbox_core::service::errors::ApiKeyServiceError;


#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ApiError {
    Generic {
        code: u32,
        message: String,
    },
    Validation {
        code: u32,
        errors: ValidationErrors,
    },
}


impl ApiError {
    pub fn new(code: u32, message: String) -> Self {
        Self::Generic { code, message }
    }

    pub fn code(&self) -> u32 {
        match self {
            Self::Generic { code, .. } => *code,
            Self::Validation { code, .. } => *code,
        }
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(403000, message.to_string())
    }

    pub fn unprocessable_entity(message: &str) -> Self {
        Self::new(422000, message.to_string())
    }

    pub fn validation_error(errors: ValidationErrors) -> Self {
        Self::Validation { code: 422001, errors }
    }

    pub fn unexpected_error(message: &str) -> Self {
        Self::new(500000, message.to_string())
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(400000, message.to_string())
    }

    pub fn invalid_content_type(content_type: &str) -> Self {
        Self::new(400001, format!("Invalid content type: {}", content_type))
    }

    pub fn invalid_data_format(message: &str) -> Self {
        Self::new(400002, message.to_string())
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(404000, message.to_string())
    }

    pub fn not_implemented() -> Self {
        Self::new(500001, "Not implemented".to_string())
    }
}


impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        ApiError::unexpected_error(&error.to_string())
    }
}

impl From<ApiKeyServiceError> for ApiError {
    fn from(error: ApiKeyServiceError) -> Self {
        match error {
            ApiKeyServiceError::KeyNotFound => ApiError::not_found("API key not found"),
            ApiKeyServiceError::KeyAlreadyExists => ApiError::bad_request("API key already exists"),
            ApiKeyServiceError::KeyExpired => ApiError::forbidden("API key expired"),
            ApiKeyServiceError::InvalidKey => ApiError::bad_request("Invalid API key"),
            ApiKeyServiceError::InsufficientScope => ApiError::forbidden("Insufficient scope"),
            ApiKeyServiceError::NamespaceNotFound => ApiError::not_found("Namespace not found"),
            ApiKeyServiceError::NamespaceAlreadyExists => ApiError::bad_request("Namespace already exists"),
            ApiKeyServiceError::TagNotFound => ApiError::not_found("Tag not found"),
            ApiKeyServiceError::TagAlreadyExists => ApiError::bad_request("Tag already exists"),
            _ => ApiError::unexpected_error(&error.to_string()),
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::validation_error(errors)
    }
}

pub type APIResult<T> = std::result::Result<T, ApiError>;
