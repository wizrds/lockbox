use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use lockbox_core::service::types::{GetNamespaceResponse, CreateNamespaceResponse};

use crate::dto::{RequestDTO, ResponseDTO};


#[derive(RequestDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct CreateNamespaceRequestDTO {
    #[validate(length(min=1, max=6))]
    pub name: String,
}

#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
#[response(status_code = axum::http::StatusCode::CREATED)]
pub struct CreateNamespaceResponseDTO {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub is_default: bool,
}

impl CreateNamespaceResponseDTO {
    pub fn from_response(response: CreateNamespaceResponse) -> Self {
        Self {
            name: response.name,
            created_at: response.created_at,
            is_default: response.is_default,
        }
    }
}

impl From<CreateNamespaceResponse> for CreateNamespaceResponseDTO {
    fn from(response: CreateNamespaceResponse) -> Self {
        CreateNamespaceResponseDTO::from_response(response)
    }
}


#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct GetNamespaceResponseDTO {
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub is_default: bool,
}

impl GetNamespaceResponseDTO {
    pub fn from_response(response: GetNamespaceResponse) -> Self {
        Self {
            name: response.name,
            created_at: response.created_at,
            is_default: response.is_default,
        }
    }
}

impl From<GetNamespaceResponse> for GetNamespaceResponseDTO {
    fn from(response: GetNamespaceResponse) -> Self {
        GetNamespaceResponseDTO::from_response(response)
    }
}
