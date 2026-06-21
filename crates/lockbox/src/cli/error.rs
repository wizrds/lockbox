#![allow(unused)]

use anyhow::Error;
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

use lockbox_core::service::errors::ApiKeyServiceError;

use crate::manifest::{to_serialized_string, ManifestFormat};


#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CliError {
    Generic {
        code: i32,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        hint: Option<String>,
    },
    Validation {
        code: i32,
        errors: ValidationErrors,
    },
}


impl CliError {
    pub fn new(code: i32, message: String) -> Self {
        Self::Generic { code, message, hint: None }
    }

    pub fn code(&self) -> i32 {
        match self {
            Self::Generic { code, .. } => *code,
            Self::Validation { code, .. } => *code,
        }
    }

    pub fn with_hint(self, hint: &str) -> Self {
        match self {
            Self::Generic { code, message, .. } => Self::Generic { code, message, hint: Some(hint.to_string()) },
            _ => self,
        }
    }

    pub fn exit(self) {
        println!("{}", to_serialized_string(&self, ManifestFormat::Json).expect("Failed to serialize error"));
        std::process::exit(self.code());
    }

    pub fn unexpected_error(message: &str) -> Self {
        Self::new(1, message.to_string())
    }

    pub fn bad_request(message: &str) -> Self {
        Self::new(2, message.to_string())
    }

    pub fn conflict(message: &str) -> Self {
        Self::new(3, message.to_string())
    }

    pub fn forbidden(message: &str) -> Self {
        Self::new(4, message.to_string())
    }

    pub fn not_found(message: &str) -> Self {
        Self::new(5, message.to_string())
    }

    pub fn validation_error(errors: ValidationErrors) -> Self {
        Self::Validation { code: 6, errors }
    }

    pub fn io_error(message: &str) -> Self {
        Self::new(7, message.to_string())
    }

    pub fn not_implemented() -> Self {
        Self::new(99, "Not implemented".to_string())
    }
}


impl From<Error> for CliError {
    fn from(error: Error) -> Self {
        CliError::unexpected_error(&error.to_string())
    }
}

impl From<ApiKeyServiceError> for CliError {
    fn from(error: ApiKeyServiceError) -> Self {
        match error {
            ApiKeyServiceError::KeyNotFound => CliError::not_found("API key not found"),
            ApiKeyServiceError::KeyAlreadyExists => CliError::conflict("API key already exists"),
            ApiKeyServiceError::KeyExpired => CliError::forbidden("API key expired"),
            ApiKeyServiceError::InvalidKey => CliError::bad_request("Invalid API key"),
            ApiKeyServiceError::InsufficientScope => CliError::forbidden("Insufficient scope"),
            ApiKeyServiceError::NamespaceNotFound => CliError::not_found("Namespace not found"),
            ApiKeyServiceError::NamespaceAlreadyExists => CliError::conflict("Namespace already exists"),
            ApiKeyServiceError::TagNotFound => CliError::not_found("Tag not found"),
            ApiKeyServiceError::TagAlreadyExists => CliError::conflict("Tag already exists"),
            _ => CliError::unexpected_error(&error.to_string()),
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        CliError::io_error(&error.to_string())
    }
}

impl From<ValidationErrors> for CliError {
    fn from(errors: ValidationErrors) -> Self {
        CliError::validation_error(errors)
    }
}
