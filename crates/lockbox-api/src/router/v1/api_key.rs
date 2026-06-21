use std::{sync::Arc, collections::HashMap};
use serde::{Deserialize, Serialize};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
    body::Body,
};
use uuid::Uuid;
use validator::Validate;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};
use chrono::{DateTime, Utc};
use jobq::JobOptions;

use lockbox_core::{
    service::{
        errors::ApiKeyServiceError, 
        api_key::{ApiKeyServiceTrait, FindApiKeyParams}
    },
    tasks::{TaskOperations, set_api_key_last_used::SetApiKeyLastUsedTask},
};

use crate::{
    dto::v1::{
        common::{PaginatedResponseDTO, ErrorResponseDTO},
        api_key::{
            CreateApiKeyRequestDTO,
            CreateApiKeyResponseDTO,
            GetApiKeyResponseDTO,
            IntrospectApiKeyRequestDTO,
            IntrospectApiKeyResponseDTO,
            SetApiKeyExpirationRequestDTO,
            SetApiKeyMetadataRequestDTO,
        },
    },
    error::ApiError,
    extractors::{Path, Query, TenantIdHeader},
    state::ApiState,
};


#[derive(IntoParams, Serialize, Deserialize, Debug, Clone, Validate)]
#[serde(default, rename_all = "camelCase")]
pub struct FindApiKeyEndpointParams {
    #[param(minimum = 1)]
    #[validate(range(min = 1))]
    pub page: Option<u64>,
    #[param(minimum = 1, maximum = 100)]
    #[validate(range(min = 1, max = 100))]
    pub per_page: Option<u64>,
    pub include_ids: Option<Vec<Uuid>>,
    pub exclude_ids: Option<Vec<Uuid>>,
    pub namespaces: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub short_keys: Option<Vec<String>>,
    pub owners: Option<Vec<String>>,
    pub revoked: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

impl Default for FindApiKeyEndpointParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
            include_ids: None,
            exclude_ids: None,
            namespaces: None,
            tags: None,
            short_keys: None,
            owners: None,
            revoked: None,
            metadata: None,
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
        .routes(routes!(create_api_key_endpoint))
        .routes(routes!(find_api_keys_endpoint))
        .routes(routes!(introspect_api_key_endpoint))
        .routes(routes!(get_api_key_endpoint))
        .routes(routes!(delete_api_key_endpoint))
        .routes(routes!(revoke_api_key_endpoint))
        .routes(routes!(rotate_api_key_endpoint))
        .routes(routes!(set_api_key_expiration_endpoint))
        .routes(routes!(set_api_key_metadata_endpoint))
}


#[utoipa::path(
    post,
    path = "",
    operation_id = "create_api_key",
    tag = "api_keys",
    request_body = CreateApiKeyRequestDTO,
    params(
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant")
    ),
    responses(
        (status = 201, description = "API key created successfully", body = CreateApiKeyResponseDTO),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 422, description = "Validation error", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn create_api_key_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    tenant_id: Option<TenantIdHeader>,
    payload: CreateApiKeyRequestDTO,
) -> impl IntoResponse {
    if let Err(err) = payload.validate() {
        return ErrorResponseDTO::from(ApiError::from(err))
            .into_response();
    }

    let service = state
        .api_key_service
        .clone();

    match service.create_api_key(
        payload.to_payload(),
        tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)
    ).await {
        Ok(response) => CreateApiKeyResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    get,
    path = "",
    operation_id = "find_api_keys",
    tag = "api_keys",
    params(
        FindApiKeyEndpointParams,
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant")
    ),
    responses(
        (status = 200, description = "API keys found", body = PaginatedResponseDTO<GetApiKeyResponseDTO>),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 422, description = "Validation error", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn find_api_keys_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Query(params): Query<FindApiKeyEndpointParams>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    if let Err(err) = params.validate() {
        return ErrorResponseDTO::from(ApiError::validation_error(err))
            .into_response();
    }

    let service = state
        .api_key_service
        .clone();

    match service.find_api_keys(FindApiKeyParams {
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(10),
        include_ids: params.include_ids,
        exclude_ids: params.exclude_ids,
        tenant_ids: tenant_id
            .map(|t| vec![t.into_inner()])
            .or(Some(vec![state.default_tenant_id.clone()])),
        namespaces: params.namespaces,
        tags: params.tags,
        short_keys: params.short_keys,
        owners: params.owners,
        revoked: params.revoked,
        metadata: params.metadata,
        created_before: params.created_before,
        created_after: params.created_after,
    }).await {
        Ok(response) => PaginatedResponseDTO::<GetApiKeyResponseDTO>::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    post,
    path = "/introspect",
    operation_id = "introspect_api_key",
    tag = "api_keys",
    request_body = IntrospectApiKeyRequestDTO,
    params(
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "API key introspection response", body = IntrospectApiKeyResponseDTO),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 422, description = "Validation error", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn introspect_api_key_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    tenant_id: Option<TenantIdHeader>,
    payload: IntrospectApiKeyRequestDTO,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();
    let job_queue = state
        .job_queue
        .clone();

    match service.introspect_api_key(
        payload.token,
        payload.scope,
        payload.tags,
        tenant_id.clone().map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)
    ).await {
        Ok(response) => {
            job_queue
                .enqueue_job(
                    JobOptions::new(TaskOperations::SetApiKeyLastUsed(
                        SetApiKeyLastUsedTask {
                            api_key_service: service.clone(),
                            id: response.key.clone().unwrap().id,
                            last_used_at: Utc::now(),
                            tenant_id: tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner),
                        }
                    ))
                )
                .await
                .expect("failed to enqueue task");

            IntrospectApiKeyResponseDTO::from(response)
                .into_response()
        },
        Err(err) => match err {
            ApiKeyServiceError::InvalidKey
            | ApiKeyServiceError::InsufficientScope
            | ApiKeyServiceError::KeyExpired => IntrospectApiKeyResponseDTO::default()
                .into_response(),
            _ => ErrorResponseDTO::from(ApiError::from(err))
                .into_response(),
        }
    }
}


#[utoipa::path(
    get,
    path = "/{id}",
    operation_id = "get_api_key",
    tag = "api_keys",
    params(
        ("id" = Uuid, Path, description = "API key ID"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "API key", body = GetApiKeyResponseDTO),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 404, description = "API key not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn get_api_key_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.get_api_key(id, tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)).await {
        Ok(response) => GetApiKeyResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    delete,
    path = "/{id}",
    operation_id = "delete_api_key",
    tag = "api_keys",
    params(
        ("id" = Uuid, Path, description = "API key ID"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 204, description = "API key deleted successfully"),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 404, description = "API key not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn delete_api_key_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.delete_api_key(id, tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)).await {
        Ok(_) => Response::builder()
            .status(204)
            .body(Body::empty())
            .unwrap(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    put,
    path = "/{id}/revoke",
    operation_id = "revoke_api_key",
    tag = "api_keys",
    params(
        ("id" = Uuid, Path, description = "API key ID"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 204, description = "API key revoked successfully"),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 404, description = "API key not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn revoke_api_key_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.revoke_api_key(id, tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)).await {
        Ok(_) => Response::builder()
            .status(204)
            .body(Body::empty())
            .unwrap(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    put,
    path = "/{id}/rotate",
    operation_id = "rotate_api_key",
    tag = "api_keys",
    params(
        ("id" = Uuid, Path, description = "API key ID"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 200, description = "API key rotated successfully", body = CreateApiKeyResponseDTO),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 404, description = "API key not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn rotate_api_key_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    tenant_id: Option<TenantIdHeader>,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.rotate_api_key(id, tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)).await {
        Ok(response) => CreateApiKeyResponseDTO::from(response)
            .into_response(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    put,
    path = "/{id}/expiration",
    operation_id = "set_api_key_expiration",
    tag = "api_keys",
    request_body = SetApiKeyExpirationRequestDTO,
    params(
        ("id" = Uuid, Path, description = "API key ID"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 204, description = "Expiration set successfully"),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 404, description = "API key not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn set_api_key_expiration_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    tenant_id: Option<TenantIdHeader>,
    payload: SetApiKeyExpirationRequestDTO,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.set_api_key_expiration(
        id,
        payload.expires_at,
        tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)
    ).await {
        Ok(_) => Response::builder()
            .status(204)
            .body(Body::empty())
            .unwrap(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}


#[utoipa::path(
    put,
    path = "/{id}/metadata",
    operation_id = "set_api_key_metadata",
    tag = "api_keys",
    request_body = SetApiKeyMetadataRequestDTO,
    params(
        ("id" = Uuid, Path, description = "API key ID"),
        ("X-Tenant-Id" = Option<TenantIdHeader>, Header, description = "The ID of the tenant"),
    ),
    responses(
        (status = 204, description = "Metadata set successfully"),
        (status = 400, description = "Bad request", body = ErrorResponseDTO),
        (status = 404, description = "API key not found", body = ErrorResponseDTO),
        (status = 500, description = "Internal server error", body = ErrorResponseDTO)
    ),
)]
async fn set_api_key_metadata_endpoint(
    Extension(state): Extension<Arc<ApiState>>,
    Path(id): Path<Uuid>,
    tenant_id: Option<TenantIdHeader>,
    payload: SetApiKeyMetadataRequestDTO,
) -> impl IntoResponse {
    let service = state
        .api_key_service
        .clone();

    match service.set_api_key_metadata(
        id,
        payload.metadata,
        tenant_id.map_or(state.default_tenant_id.clone(), TenantIdHeader::into_inner)
    ).await {
        Ok(_) => Response::builder()
            .status(204)
            .body(Body::empty())
            .unwrap(),
        Err(err) => ErrorResponseDTO::from(ApiError::from(err))
            .into_response(),
    }
}
