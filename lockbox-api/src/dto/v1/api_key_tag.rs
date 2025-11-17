use serde::{Deserialize, Serialize};
use validator::Validate;
use utoipa::ToSchema;
use chrono::{DateTime, Utc};

use lockbox_core::service::types::{GetTagResponse, CreateTagResponse};

use crate::dto::{RequestDTO, ResponseDTO};


#[derive(RequestDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct CreateTagRequestDTO {
    #[validate(length(min=1, max=6))]
    pub name: String,
}

#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
#[response(status_code = axum::http::StatusCode::CREATED)]
pub struct CreateTagResponseDTO {
    pub namespace: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl CreateTagResponseDTO {
    pub fn from_response(response: CreateTagResponse) -> Self {
        Self {
            namespace: response.namespace,
            name: response.name,
            created_at: response.created_at,
        }
    }
}

impl From<CreateTagResponse> for CreateTagResponseDTO {
    fn from(response: CreateTagResponse) -> Self {
        CreateTagResponseDTO::from_response(response)
    }
}


#[derive(ResponseDTO, Serialize, Deserialize, Debug, Clone, Validate, ToSchema)]
pub struct GetTagResponseDTO {
    pub namespace: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl GetTagResponseDTO {
    pub fn from_response(response: GetTagResponse) -> Self {
        Self {
            namespace: response.namespace,
            name: response.name,
            created_at: response.created_at,
        }
    }
}

impl From<GetTagResponse> for GetTagResponseDTO {
    fn from(response: GetTagResponse) -> Self {
        GetTagResponseDTO::from_response(response)
    }
}
