use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("request error: {0}")]
    Request(#[from] reqwest_middleware::Error),

    #[error("deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("query params error: {0}")]
    QueryParams(String),

    #[error("API error {status}: {body:?}")]
    Api { status: u16, body: ErrorBody },

    #[error("session closed before response was received")]
    SessionClosed,

    #[error("session returned an unexpected response type")]
    UnexpectedResponse,

    #[error("session error: {message}")]
    Session {
        message: String,
        code: Option<u32>,
        fatal: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ErrorBody {
    // Validation must come first: serde tries untagged variants in order and Validation is more specific.
    Validation {
        code: u32,
        fields: Vec<ValidationErrorField>,
    },
    Generic {
        code: u32,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<Value>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorField {
    pub field: String,
    pub errors: Vec<ValidationErrorFieldDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorFieldDetail {
    pub code: String,
    pub message: Option<String>,
    pub params: Option<Value>,
}
