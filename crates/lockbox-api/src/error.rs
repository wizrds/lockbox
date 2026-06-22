use anyhow::Error;
use serde::{Deserialize, Serialize};
use validator::ValidationErrors;
use axum::extract::{
    rejection::{
        JsonRejection,
        PathRejection,
        QueryRejection,
    },
    path::ErrorKind,
};

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
    pub fn new(code: u32, message: impl Into<String>) -> Self {
        Self::Generic { code, message: message.into() }
    }

    pub fn code(&self) -> u32 {
        match self {
            Self::Generic { code, .. } => *code,
            Self::Validation { code, .. } => *code,
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(403000, message)
    }

    pub fn unprocessable_entity(message: impl Into<String>) -> Self {
        Self::new(422000, message)
    }

    pub fn validation_error(errors: ValidationErrors) -> Self {
        Self::Validation { code: 422001, errors }
    }

    pub fn unexpected_error(message: impl Into<String>) -> Self {
        Self::new(500000, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400000, message)
    }

    pub fn invalid_content_type(content_type: impl Into<String>) -> Self {
        Self::new(400001, format!("Invalid content type: {}", content_type.into()))
    }

    pub fn invalid_data_format(message: impl Into<String>) -> Self {
        Self::new(400002, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404000, message)
    }

    pub fn not_implemented() -> Self {
        Self::new(500001, "Not implemented")
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

impl From<PathRejection> for ApiError {
    fn from(error: PathRejection) -> Self {
        match error {
            PathRejection::FailedToDeserializePathParams(inner) => {
                match inner.into_kind() {
                    ErrorKind::WrongNumberOfParameters { got, expected } => ApiError::bad_request(&format!(
                        "Wrong number of path parameters: got {}, expected {}",
                        got, expected
                    )),
                    ErrorKind::ParseErrorAtKey { key, .. } => ApiError::bad_request(&format!(
                        "Failed to parse path parameter '{}'", key
                    )),
                    ErrorKind::ParseErrorAtIndex { index, .. } => ApiError::bad_request(&format!(
                        "Failed to parse path parameter at index {}", index
                    )),
                    ErrorKind::ParseError { value, expected_type } => ApiError::bad_request(&format!(
                        "Failed to parse path parameter value '{}' as {}", value, expected_type
                    )),
                    ErrorKind::InvalidUtf8InPathParam { key } => ApiError::bad_request(&format!(
                        "Invalid UTF-8 in path parameter '{}'", key
                    )),
                    ErrorKind::UnsupportedType { name } => ApiError::bad_request(&format!(
                        "Unsupported type for path parameter '{}'", name
                    )),
                    ErrorKind::Message(msg) => ApiError::bad_request(&msg),
                    ErrorKind::DeserializeError { message, .. } => ApiError::bad_request(message.as_str()),
                    _ => ApiError::unexpected_error("Failed to extract path parameters"),
                }
            },
            PathRejection::MissingPathParams(error) => ApiError::unexpected_error(error.to_string()),
            _ => ApiError::unexpected_error("Failed to extract path parameters"),
        }
    }
}

impl From<QueryRejection> for ApiError {
    fn from(error: QueryRejection) -> Self {
        match error {
            QueryRejection::FailedToDeserializeQueryString(_) => ApiError::bad_request("Invalid query string"),
            _ => ApiError::bad_request("Unexpected query rejection"),
        }
    }
}

impl From<JsonRejection> for ApiError {
    fn from(error: JsonRejection) -> Self {
        match error {
            JsonRejection::JsonDataError(err) => ApiError::unprocessable_entity(err.to_string()),
            JsonRejection::JsonSyntaxError(_) => ApiError::invalid_data_format("Invalid JSON syntax"),
            JsonRejection::MissingJsonContentType(_) => ApiError::invalid_content_type("Missing JSON content type"),
            _ => ApiError::bad_request("Unexpected JSON rejection"),
        }
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(errors: ValidationErrors) -> Self {
        ApiError::validation_error(errors)
    }
}

pub type APIResult<T> = std::result::Result<T, ApiError>;
