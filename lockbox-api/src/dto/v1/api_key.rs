use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use lockbox_core::service::types::{
    GetApiKeyResponse,
    CreateApiKeyPayload,
    CreateApiKeyResponse,
    IntrospectApiKeyResponse
};

use crate::dto::{RequestDTO, ResponseDTO, v1::utils::ensure_future_date};


#[derive(RequestDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct CreateApiKeyRequestDTO {
    #[validate(length(max=255))]
    pub owner: String,
    #[validate(length(max=255))]
    pub scope: Option<String>,
    #[validate(length(max=6))]
    pub tag: Option<String>,
    #[validate(custom(function = ensure_future_date))]
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Option<HashMap<String, String>>,
}

impl CreateApiKeyRequestDTO {
    pub fn to_payload(self) -> CreateApiKeyPayload {
        CreateApiKeyPayload {
            owner: self.owner,
            scope: self.scope,
            expires_at: self.expires_at,
            tag: self.tag,
            metadata: self.metadata.unwrap_or_default(),
        }
    }
}


#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
#[response(status_code = axum::http::StatusCode::CREATED)]
pub struct CreateApiKeyResponseDTO {
    pub id: String,
    pub namespace: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub owner: String,
    pub scope: Option<String>,
    pub tag: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl CreateApiKeyResponseDTO {
    pub fn from_response(response: CreateApiKeyResponse) -> Self {
        Self {
            id: response.id.hyphenated().to_string(),
            namespace: response.namespace,
            key: response.key,
            created_at: response.created_at,
            owner: response.owner,
            scope: response.scope,
            tag: response.tag,
            expires_at: response.expires_at,
            metadata: response.metadata,
        }
    }
}

impl From<CreateApiKeyResponse> for CreateApiKeyResponseDTO {
    fn from(response: CreateApiKeyResponse) -> Self {
        CreateApiKeyResponseDTO::from_response(response)
    }
}


#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct GetApiKeyResponseDTO {
    pub id: String,
    pub namespace: String,
    pub short_key: String,
    pub created_at: DateTime<Utc>,
    pub owner: String,
    pub scope: Option<String>,
    pub tag: Option<String>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}


impl GetApiKeyResponseDTO {
    pub fn from_response(response: GetApiKeyResponse) -> Self {
        Self {
            id: response.id.hyphenated().to_string(),
            namespace: response.namespace,
            short_key: response.short_key,
            created_at: response.created_at,
            owner: response.owner,
            scope: response.scope,
            tag: response.tag,
            revoked: response.revoked,
            revoked_at: response.revoked_at,
            expires_at: response.expires_at,
            last_used_at: response.last_used_at,
            metadata: response.metadata,
        }
    }
}


impl From<GetApiKeyResponse> for GetApiKeyResponseDTO {
    fn from(response: GetApiKeyResponse) -> Self {
        GetApiKeyResponseDTO::from_response(response)
    }
}


#[derive(RequestDTO, Serialize, Deserialize, Default, Debug, Clone, Validate, ToSchema)]
#[serde(default)]
pub struct IntrospectApiKeyRequestDTO {
    #[validate(length(max=255))]
    pub token: String,
    #[validate(length(max=255))]
    pub scope: Option<String>,
    #[validate(length(max=1))]
    pub tags: Option<Vec<Option<String>>>,
}


#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct IntrospectApiKeyResponseDTO {
    pub valid: bool,
    pub key: Option<GetApiKeyResponseDTO>,
}

impl IntrospectApiKeyResponseDTO {
    pub fn from_response(response: IntrospectApiKeyResponse) -> Self {
        Self {
            valid: response.valid,
            key: response.key.map(GetApiKeyResponseDTO::from),
        }
    }
}

impl Default for IntrospectApiKeyResponseDTO {
    fn default() -> Self {
        Self {
            valid: false,
            key: None,
        }
    }
}

impl From<IntrospectApiKeyResponse> for IntrospectApiKeyResponseDTO {
    fn from(response: IntrospectApiKeyResponse) -> Self {
        IntrospectApiKeyResponseDTO::from_response(response)
    }
}


#[derive(RequestDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct SetApiKeyExpirationRequestDTO {
    #[validate(custom(function = ensure_future_date))]
    pub expires_at: DateTime<Utc>,
}


#[derive(RequestDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct SetApiKeyMetadataRequestDTO {
    pub metadata: HashMap<String, String>,
}