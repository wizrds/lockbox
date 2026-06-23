use reqwest::Response;

use crate::{
    base::{ApiClient, Call},
    types::{
        common::page::Page,
        tags::{
            CreateTagRequest,
            CreateTagResponse,
            GetTagResponse,
            FindTagsParams,
        }
    },
};


/// The Tags client.
pub struct TagsClient<'c> {
    client: &'c ApiClient,
}

impl<'c> TagsClient<'c> {
    pub(crate) fn new(client: &'c ApiClient) -> Self {
        Self { client }
    }

    /// Create a new tag.
    pub fn create_tag(&self, request: &CreateTagRequest) -> Call<'_, Response, CreateTagResponse> {
        Call::post(self.client, "/v1/tags")
            .body(request)
            .json()
    }

    /// Get a tag by name.
    pub fn get_tag(&self, name: &str) -> Call<'_, Response, GetTagResponse> {
        Call::get(self.client, format!("/v1/tags/{}", name))
            .json()
    }

    /// Find tags with optional filters.
    pub fn find_tags(&self, params: Option<&FindTagsParams>) -> Call<'_, Response, Page<GetTagResponse>> {
        Call::get(self.client, "/v1/tags")
            .maybe_params(params)
            .json()
    }

    /// Delete a tag by name.
    pub fn delete_tag(&self, name: &str) -> Call<'_, Response, ()> {
        Call::delete(self.client, format!("/v1/tags/{}", name))
            .empty()
    }
}
