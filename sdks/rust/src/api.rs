use reqwest::Response;

use crate::{
    base::{ApiClient, Call, ClientConfig},
    clients::{
        namespaces::NamespacesClient,
        tags::TagsClient,
        api_keys::ApiKeysClient,
    },
    types::common::ping::PingResponse,
};


/// The Lockbox API client.
#[derive(Clone)]
pub struct LockboxApiClient {
    client: ApiClient,
}

impl LockboxApiClient {
    pub fn new(config: ClientConfig) -> Self {
        Self {
            client: ApiClient::from_config(config),
        }
    }

    /// Pings the API, returning its name and version.
    pub fn ping(&self) -> Call<'_, Response, PingResponse> {
        Call::get(&self.client, "/")
            .json()
    }

    /// Returns a client for namespace operations.
    pub fn namespaces(&self) -> NamespacesClient<'_> {
        NamespacesClient::new(&self.client)
    }

    /// Returns a client for tag operations.
    pub fn tags(&self) -> TagsClient<'_> {
        TagsClient::new(&self.client)
    }

    /// Returns a client for API key operations.
    pub fn api_keys(&self) -> ApiKeysClient<'_> {
        ApiKeysClient::new(&self.client)
    }
}
