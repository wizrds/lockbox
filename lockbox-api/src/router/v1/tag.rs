use serde::{Deserialize, Serialize};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
    body::Body,
    http::StatusCode,
};
use std::sync::Arc;
use validator::Validate;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use chrono::{DateTime, Utc};

use lockbox_core::service::api_key::{ApiKeyServiceTrait, FindApiKeyTagParams};

use crate::{
    dto::v1::{
        common::{PaginatedResponseDTO, ErrorResponseDTO},
        api_key_tag::{
            GetTagResponseDTO,
            CreateTagRequestDTO,
            CreateTagResponseDTO,
        }
    },
    error::ApiError,
    extractors::{Path, Query, TenantIdHeader},
    state::ApiState,
};


#[derive(IntoParams, Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(default, rename_all = "camelCase")]
pub struct FindApiKeyTagEndpointParams {
    #[param(minimum = 1)]
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[param(minimum = 1, maximum = 100)]
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub names: Option<Vec<String>>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

impl Default for FindApiKeyTagEndpointParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
            names: None,
            created_before: None,
            created_after: None,
        }
    }
}


pub fn router() -> OpenApiRouter {
    OpenApiRouter::new()
        // The endpoint functions need to be registered separately
        // as opposed to a single `routes!` macro call
        // else it will panic saying overlapping method route
        // despite having different paths
        .routes(routes!(create_tag_endpoint))
        .routes(routes!(find_tags_endpoint))
        .routes(routes!(get_tag_endpoint))
        .routes(routes!(delete_tag_endpoint))
}


#[utoipa::path(
    post,
    path = "",
    operation_id = "create_tag",
    tag = "tags",
    params(
        ("namespace" = String, Path, description = "Namespace name"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    request_body = CreateTagRequestDTO,
    responses(
        (status = 201, description = "Tag created", body = CreateTagResponseDTO),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn create_tag_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(namespace): Path<String>,
    tenant_id: Option<TenantIdHeader>,
    payload: CreateTagRequestDTO,
) -> impl IntoResponse {
    if let Err(err) = payload.validate() {
        return ErrorResponseDTO::from(ApiError::validation_error(err))
            .into_response();
    }

    let service = state
        .api_key_service
        .clone();

    match service.create_tag(
        payload.name,
        namespace,
        tenant_id.map_or(state.default_tenant_id.clone(), |t| t.into_inner()),
    ).await {
        Ok(response) => CreateTagResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    get,
    path = "",
    operation_id = "find_tags",
    tag = "tags",
    params(
        FindApiKeyTagEndpointParams,
        ("namespace" = String, Path, description = "Namespace name"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "Tags found", body = PaginatedResponseDTO<GetTagResponseDTO>),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn find_tags_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Query(params): Query<FindApiKeyTagEndpointParams>,
    Path(namespace): Path<String>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    if let Err(err) = params.validate() {
        return ErrorResponseDTO::from(ApiError::validation_error(err))
            .into_response();
    }

    let service = state
        .api_key_service
        .clone();

    match service.find_tags(FindApiKeyTagParams {
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(10),
        tenant_ids: tenant_id
            .map(|t| vec![t.into_inner()])
            .or(Some(vec![state.default_tenant_id.clone()])),
        namespaces: Some(vec![namespace]),
        names: params.names,
        created_before: params.created_before,
        created_after: params.created_after,
    }).await {
        Ok(response) => PaginatedResponseDTO::<GetTagResponseDTO>::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    get,
    path = "/{name}",
    operation_id = "get_tag",
    tag = "tags",
    params(
        ("namespace" = String, Path, description = "Namespace name"),
        ("name" = String, Path, description = "Tag name"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "Tag", body = GetTagResponseDTO),
        (status = 404, description = "Tag not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn get_tag_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path((namespace, name)): Path<(String, String)>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.get_tag(
        name,
        namespace,
        tenant_id.map_or(state.default_tenant_id.clone(), |t| t.into_inner()),
    ).await {
        Ok(response) => GetTagResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    delete,
    path = "/{name}",
    operation_id = "delete_tag",
    tag = "tags",
    params(
        ("namespace" = String, Path, description = "Namespace name"),
        ("name" = String, Path, description = "Tag name"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 204, description = "Tag deleted"),
        (status = 404, description = "Tag not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn delete_tag_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path((namespace, name)): Path<(String, String)>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.delete_tag(
        name,
        namespace,
        tenant_id.map_or(state.default_tenant_id.clone(), |t| t.into_inner()),
    ).await {
        Ok(_) => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}