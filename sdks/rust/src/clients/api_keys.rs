use reqwest::Response;
use uuid::Uuid;

use crate::{
    base::{ApiClient, Call},
    types::{
        common::page::Page,
        api_keys::{
            CreateApiKeyRequest,
            CreateApiKeyResponse,
            GetApiKeyResponse,
            IntrospectApiKeyRequest,
            IntrospectApiKeyResponse,
            SetApiKeyExpirationRequest,
            SetApiKeyMetadataRequest,
            FindApiKeysParams,
        }
    },
};


/// The API Keys client.
pub struct ApiKeysClient<'c> {
    client: &'c ApiClient,
}

impl<'c> ApiKeysClient<'c> {
    pub(crate) fn new(client: &'c ApiClient) -> Self {
        Self { client }
    }

    /// Create a new API key.
    pub fn create_api_key(&self, request: &CreateApiKeyRequest) -> Call<'_, Response, CreateApiKeyResponse> {
        Call::post(self.client, "/v1/api_keys")
            .body(request)
            .json()
    }

    /// Get an API key by ID.
    pub fn get_api_key(&self, id: &Uuid) -> Call<'_, Response, GetApiKeyResponse> {
        Call::get(self.client, format!("/v1/api_keys/{}", id))
            .json()
    }

    /// Introspect an API key.
    pub fn introspect_api_key(&self, request: &IntrospectApiKeyRequest) -> Call<'_, Response, IntrospectApiKeyResponse> {
        Call::post(self.client, "/v1/api_keys/introspect")
            .body(request)
            .json()
    }

    /// Revoke an API key.
    pub fn revoke_api_key(&self, id: &Uuid) -> Call<'_, Response, ()> {
        Call::put(self.client, format!("/v1/api_keys/{}/revoke", id))
            .empty()
    }

    /// Rotate an API key.
    pub fn rotate_api_key(&self, id: &Uuid) -> Call<'_, Response, CreateApiKeyResponse> {
        Call::put(self.client, format!("/v1/api_keys/{}/rotate", id))
            .json()
    }

    /// Set the expiration for an API key.
    pub fn set_api_key_expiration(&self, id: &Uuid, request: &SetApiKeyExpirationRequest) -> Call<'_, Response, ()> {
        Call::put(self.client, format!("/v1/api_keys/{}/expiration", id))
            .body(request)
            .empty()
    }

    /// Set the metadata for an API key.
    pub fn set_api_key_metadata(&self, id: &Uuid, request: &SetApiKeyMetadataRequest) -> Call<'_, Response, ()> {
        Call::put(self.client, format!("/v1/api_keys/{}/metadata", id))
            .body(request)
            .empty()
    }

    /// Find API keys with optional filters.
    pub fn find_api_keys(&self, params: Option<&FindApiKeysParams>) -> Call<'_, Response, Page<GetApiKeyResponse>> {
        Call::get(self.client, "/v1/api_keys")
            .maybe_params(params)
            .json()
    }

    /// Delete an API key by ID.
    pub fn delete_api_key(&self, id: &Uuid) -> Call<'_, Response, ()> {
        Call::delete(self.client, format!("/v1/api_keys/{}", id))
            .empty()
    }
}
