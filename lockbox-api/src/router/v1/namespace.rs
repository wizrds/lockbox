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

use lockbox_core::service::api_key::{ApiKeyServiceTrait, FindApiKeyNamespaceParams};

use crate::{
    router::v1::tag::router as tag_router,
    dto::v1::{
        common::{PaginatedResponseDTO, ErrorResponseDTO},
        api_key_namespace::{
            GetNamespaceResponseDTO,
            CreateNamespaceRequestDTO,
            CreateNamespaceResponseDTO,
        },
    },
    error::ApiError,
    extractors::{Path, Query, TenantIdHeader},
    state::ApiState,
};


#[derive(IntoParams, Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(default, rename_all = "camelCase")]
pub struct FindApiKeyNamespaceEndpointParams {
    #[param(minimum = 1)]
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[param(minimum = 1, maximum = 100)]
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub names: Option<Vec<String>>,
    pub is_default: Option<bool>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

impl Default for FindApiKeyNamespaceEndpointParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
            names: None,
            is_default: None,
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
        .routes(routes!(create_namespace_endpoint))
        .routes(routes!(find_namespaces_endpoint))
        .routes(routes!(get_namespace_endpoint))
        .routes(routes!(delete_namespace_endpoint))
        .nest("/{namespace}/tags", tag_router())
}


#[utoipa::path(
    post,
    path = "",
    operation_id = "create_namespace",
    tag = "namespaces",
    request_body = CreateNamespaceRequestDTO,
    params(
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 201, description = "Namespace created", body = CreateNamespaceResponseDTO),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn create_namespace_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    tenant_id: Option<TenantIdHeader>,
    payload: CreateNamespaceRequestDTO,
) -> impl IntoResponse {
    if let Err(err) = payload.validate() {
        return ErrorResponseDTO::from(ApiError::validation_error(err))
            .into_response();
    }

    let service = state
        .api_key_service
        .clone();

    match service.create_namespace(
        payload.name,
        tenant_id.map_or(state.default_tenant_id.clone(), |t| t.into_inner()),
    ).await {
        Ok(response) => CreateNamespaceResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    get,
    path = "",
    operation_id = "find_namespaces",
    tag = "namespaces",
    params(
        FindApiKeyNamespaceEndpointParams,
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "Namespaces found", body = PaginatedResponseDTO<GetNamespaceResponseDTO>),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn find_namespaces_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Query(params): Query<FindApiKeyNamespaceEndpointParams>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    if let Err(err) = params.validate() {
        return ErrorResponseDTO::from(ApiError::validation_error(err))
            .into_response();
    }

    let service = state
        .api_key_service
        .clone();

    match service.find_namespaces(FindApiKeyNamespaceParams {
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(10),
        tenant_ids: tenant_id
            .map(|t| vec![t.into_inner()])
            .or(Some(vec![state.default_tenant_id.clone()])),
        names: params.names,
        is_default: params.is_default,
        created_before: params.created_before,
        created_after: params.created_after,
    }).await {
        Ok(response) => PaginatedResponseDTO::<GetNamespaceResponseDTO>::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    get,
    path = "/{name}",
    operation_id = "get_namespace",
    tag = "namespaces",
    params(
        ("name" = String, Path, description = "Namespace name"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "Namespace", body = GetNamespaceResponseDTO),
        (status = 404, description = "Namespace not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn get_namespace_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(name): Path<String>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.get_namespace(name, tenant_id.map_or(state.default_tenant_id.clone(), |t| t.into_inner())).await {
        Ok(response) => GetNamespaceResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    delete,
    path = "/{name}",
    operation_id = "delete_namespace",
    tag = "namespaces",
    params(
        ("name" = String, Path, description = "Namespace name"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 204, description = "Namespace deleted"),
        (status = 404, description = "Namespace not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    )
)]
async fn delete_namespace_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(name): Path<String>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.delete_namespace(name, tenant_id.map_or(state.default_tenant_id.clone(), |t| t.into_inner())).await {
        Ok(_) => Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}