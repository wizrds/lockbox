use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, path::ErrorKind, rejection::PathRejection},
    http::{request::Parts, header::AUTHORIZATION},
};
use serde::de::DeserializeOwned;
use serde_qs::Config as QsConfig;
use utoipa::ToSchema;

use crate::dto::v1::common::ErrorResponseDTO;
use crate::error::ApiError;


#[derive(Debug, Clone, ToSchema)]
pub struct TenantIdHeader(pub String);

impl TenantIdHeader {
    pub fn new(tenant_id: String) -> Self {
        Self(tenant_id)
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn extract_from_parts(parts: &Parts) -> Option<TenantIdHeader> {
        match parts.headers.get("x-tenant-id") {
            Some(value) => Some(Self(value.to_str().ok()?.to_string())),
            None => None,
        }
    }
}

impl<S> FromRequestParts<S> for TenantIdHeader
where
    S: Send + Sync
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match Self::extract_from_parts(parts) {
            Some(tenant_id) => Ok(tenant_id),
            None => Err(ErrorResponseDTO::from(
                ApiError::bad_request("Missing X-Tenant-ID header")
            )),
        }
    }
}

impl<S> OptionalFromRequestParts<S> for TenantIdHeader
where
    S: Send + Sync,
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Self::Rejection> {
        Ok(Self::extract_from_parts(parts))
    }
}


#[derive(Debug, Clone, ToSchema)]
pub struct ApiKeyHeader(pub String);

impl ApiKeyHeader {
    pub fn new(api_key: String) -> Self {
        Self(api_key)
    }

    pub fn as_inner(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }

    pub fn extract_from_parts(parts: &Parts) -> Option<ApiKeyHeader> {
        match parts.headers.get(AUTHORIZATION) {
            Some(value) => Some(Self(value.to_str().ok()?.to_string())),
            None => None,
        }
    }
}

impl<S> FromRequestParts<S> for ApiKeyHeader
where
    S: Send + Sync
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match Self::extract_from_parts(parts) {
            Some(api_key) => Ok(api_key),
            None => Err(ErrorResponseDTO::from(
                ApiError::bad_request("Missing Authorization header")
            )),
        }
    }
}

impl<S> OptionalFromRequestParts<S> for ApiKeyHeader
where
    S: Send + Sync,
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Option<Self>, Self::Rejection> {
        Ok(Self::extract_from_parts(parts))
    }
}


#[derive(Debug, Clone, ToSchema)]
pub struct Path<T>(pub T);

impl<S, T> FromRequestParts<S> for Path<T>
where 
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match <axum::extract::Path<T> as FromRequestParts<S>>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => match rejection {
                PathRejection::FailedToDeserializePathParams(inner) => {
                    match inner.into_kind() {
                        ErrorKind::WrongNumberOfParameters { got, expected } => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request(&format!(
                                    "Wrong number of path parameters: got {}, expected {}",
                                    got, expected
                                ))
                            )
                        ),
                        ErrorKind::ParseErrorAtKey { key, .. } => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request(&format!(
                                    "Failed to parse path parameter '{}'", key
                                ))
                            ),
                        ),
                        ErrorKind::ParseErrorAtIndex { index, .. } => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request(&format!(
                                    "Failed to parse path parameter at index {}", index
                                ))
                            ),
                        ),
                        ErrorKind::ParseError { .. } => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request("Failed to parse path parameters")
                            ),
                        ),
                        ErrorKind::InvalidUtf8InPathParam { key } => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request(&format!(
                                    "Invalid UTF-8 in path parameter '{}'", key
                                ))
                            ),
                        ),
                        ErrorKind::UnsupportedType { .. } => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request("Unsupported type for path parameter")
                            ),
                        ),
                        ErrorKind::Message(msg) => Err(
                            ErrorResponseDTO::from(
                                ApiError::bad_request(&msg)
                            ),
                        ),
                        _ => Err(
                            ErrorResponseDTO::from(
                                ApiError::unexpected_error("Failed to extract path parameters")
                            ),
                        ),
                    }
                },
                PathRejection::MissingPathParams(error) => Err(
                    ErrorResponseDTO::from(
                        ApiError::unexpected_error(&error.to_string())
                    )
                ),
                _ => Err(
                    ErrorResponseDTO::from(
                        ApiError::unexpected_error("Failed to extract path parameters")
                    )
                ),
            }
        }
    }
}


#[derive(Debug, Clone, ToSchema)]
pub struct Query<T>(pub T);

impl<S, T> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let query = parts.uri.query().unwrap_or("");
        let config = QsConfig::new(5, false);

        config
            .deserialize_str::<T>(query)
            .map(|query| Query(query))
            .map_err(|err| ErrorResponseDTO::from(
                ApiError::bad_request(err.to_string().as_str())
            ))
    }
}


pub struct OptionalQuery<T>(pub Option<T>);

impl<S, T> FromRequestParts<S> for OptionalQuery<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = ErrorResponseDTO;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match parts.uri.query() {
            Some(_) => Query::from_request_parts(parts, state)
                .await
                .map(|q| OptionalQuery(Some(q.0))),
            None => Ok(OptionalQuery(None)),
        }
    }
}