use reqwest::Response;

use crate::{
    base::{ApiClient, Call},
    types::namespaces::{
        CreateNamespaceRequest,
        CreateNamespaceResponse,
        GetNamespaceResponse,
    },
};


/// The Namespaces client.
pub struct NamespacesClient<'c> {
    client: &'c ApiClient,
}

impl<'c> NamespacesClient<'c> {
    pub(crate) fn new(client: &'c ApiClient) -> Self {
        Self { client }
    }

    /// Create a new namespace.
    pub fn create_namespace(&self, request: &CreateNamespaceRequest) -> Call<'_, Response, CreateNamespaceResponse> {
        Call::post(self.client, "/v1/namespaces")
            .body(request)
            .json()
    }

    /// Get a namespace by name.
    pub fn get_namespace(&self, name: &str) -> Call<'_, Response, GetNamespaceResponse> {
        Call::get(self.client, format!("/v1/namespaces/{}", name))
            .json()
    }

    /// Delete a namespace by name.
    pub fn delete_namespace(&self, name: &str) -> Call<'_, Response, ()> {
        Call::delete(self.client, format!("/v1/namespaces/{}", name))
            .empty()
    }
}
