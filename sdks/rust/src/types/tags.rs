use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagRequest {
    /// The name of the tag to create. Must be unique within the namespace and between 1 and 6 characters.
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTagResponse {
    /// The namespace the tag belongs to.
    pub namespace: String,
    /// The name of the created tag.
    pub name: String,
    /// The timestamp when the tag was created.
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTagResponse {
    /// The namespace the tag belongs to.
    pub namespace: String,
    /// The name of the tag.
    pub name: String,
    /// The timestamp when the tag was created.
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FindTagsParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<Utc>>,
}

impl FindTagsParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn page(mut self, page: u64) -> Self {
        self.page = Some(page);
        self
    }

    pub fn per_page(mut self, per_page: u64) -> Self {
        self.per_page = Some(per_page);
        self
    }

    pub fn names(mut self, names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.names = Some(names.into_iter().map(Into::into).collect());
        self
    }

    pub fn created_after(mut self, created_after: DateTime<Utc>) -> Self {
        self.created_after = Some(created_after);
        self
    }

    pub fn created_before(mut self, created_before: DateTime<Utc>) -> Self {
        self.created_before = Some(created_before);
        self
    }
}
