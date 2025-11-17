use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use axum::{
    http::StatusCode,
    response::Json,
    response::IntoResponse,
    response::Response,
};
use utoipa::ToSchema;

use lockbox_core::database::paginate::Page;

use crate::error::ApiError;


#[derive(Serialize, Deserialize, ToSchema, Debug)]
#[serde(untagged)]
pub enum ErrorResponseDTO {
    Generic {
        code: u32,
        message: String,
    },
    Validation {
        code: u32,
        fields: Vec<ValidationErrorFieldDTO>,
    },
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ValidationErrorFieldDTO {
    pub field: String,
    pub errors: Vec<ValidationErrorFieldDetailDTO>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ValidationErrorFieldDetailDTO {
    pub code: String,
    pub message: Option<String>,
    pub params: Value,
}

impl From<ApiError> for ErrorResponseDTO {
    fn from(error: ApiError) -> Self {
        match error {
            ApiError::Generic { code, message } => ErrorResponseDTO::Generic { code, message },
            ApiError::Validation { code, errors } => {
                ErrorResponseDTO::Validation {
                    code,
                    fields: errors
                        .field_errors()
                        .into_iter()
                        .map(|(field, field_errors)| {
                            ValidationErrorFieldDTO {
                                field: field.to_string(),
                                errors: field_errors
                                    .iter()
                                    .map(|error| {
                                        ValidationErrorFieldDetailDTO {
                                            code: error
                                                .code
                                                .to_string(),
                                            message: error
                                                .message
                                                .as_ref()
                                                .map(|msg| msg.to_string()),
                                            params: to_value(&error.params)
                                                .unwrap_or(Value::Null),
                                        }
                                    })
                                    .collect(),
                            }
                        })
                        .collect(),
                }
            }
        }
    }
}

impl IntoResponse for ErrorResponseDTO {
    fn into_response(self) -> Response {
        let status = match self {
            ErrorResponseDTO::Generic { code, .. } => {
                StatusCode::from_u16((code / 1000) as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            ErrorResponseDTO::Validation { code, .. } => {
                StatusCode::from_u16((code / 1000) as u16).unwrap_or(StatusCode::BAD_REQUEST)
            }
        };

        (status, Json(self)).into_response()
    }
}


#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct PaginatedResponseDTO<T> {
    pub items: Vec<T>,
    pub count: u64,
    pub next_page: Option<u64>,
    pub previous_page: Option<u64>,
}

impl<T> PaginatedResponseDTO<T> {
    pub fn builder(items: Vec<T>) -> PaginatedResponseBuilder<T> {
        PaginatedResponseBuilder::new(items)
    }
}

pub struct PaginatedResponseBuilder<T> {
    items: Vec<T>,
    count: u64,
    next_page: Option<u64>,
    previous_page: Option<u64>,
}

impl<T> PaginatedResponseBuilder<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            count: 0,
            next_page: None,
            previous_page: None,
        }
    }

    pub fn with_count(mut self, count: u64) -> Self {
        self.count = count;
        self
    }

    pub fn with_next_page(mut self, next_page: Option<u64>) -> Self {
        self.next_page = next_page;
        self
    }

    pub fn with_previous_page(mut self, previous_page: Option<u64>) -> Self {
        self.previous_page = previous_page;
        self
    }

    pub fn build(self) -> PaginatedResponseDTO<T> {
        PaginatedResponseDTO {
            items: self.items,
            count: self.count,
            next_page: self.next_page,
            previous_page: self.previous_page,
        }
    }
}

impl<T> IntoResponse for PaginatedResponseDTO<T>
where
    T: Serialize + ToSchema,
{
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl<T, D> From<Page<T>> for PaginatedResponseDTO<D>
where
    D: From<T>,
{
    fn from(page: Page<T>) -> PaginatedResponseDTO<D> {
        PaginatedResponseDTO::builder(
            page
            .items
            .into_iter()
            .map(D::from)
            .collect::<Vec<D>>()
        )
            .with_count(page.count)
            .with_next_page(page.next_page)
            .with_previous_page(page.previous_page)
            .build()
    }
}