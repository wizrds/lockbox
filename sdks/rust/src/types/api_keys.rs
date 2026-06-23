use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    /// Optional ID of the key
    pub id: Option<Uuid>,
    /// The owner of the API key.
    pub owner: String,
    /// The scope granted to the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The tag to associate with the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// The timestamp at which the API key expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Arbitrary metadata to associate with the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    /// The ID of the created API key.
    pub id: Uuid,
    /// The namespace the API key belongs to.
    pub namespace: String,
    /// The full API key value. Only returned at creation time.
    pub key: String,
    /// The timestamp when the API key was created.
    pub created_at: DateTime<Utc>,
    /// The owner of the API key.
    pub owner: String,
    /// The scope granted to the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The tag associated with the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// The timestamp at which the API key expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Arbitrary metadata associated with the API key.
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetApiKeyResponse {
    /// The ID of the API key.
    pub id: Uuid,
    /// The namespace the API key belongs to.
    pub namespace: String,
    /// A non-secret, truncated portion of the API key.
    pub short_key: String,
    /// The timestamp when the API key was created.
    pub created_at: DateTime<Utc>,
    /// The owner of the API key.
    pub owner: String,
    /// The scope granted to the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The tag associated with the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Whether the API key has been revoked.
    pub revoked: bool,
    /// The timestamp when the API key was revoked, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<Utc>>,
    /// The timestamp at which the API key expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// The timestamp the API key was last used, if ever.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,
    /// Arbitrary metadata associated with the API key.
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntrospectApiKeyRequest {
    /// The API key value to introspect.
    pub token: String,
    /// The scope the key is expected to satisfy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// The tags the key is expected to satisfy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Option<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntrospectApiKeyResponse {
    /// Whether the introspected key is valid.
    pub valid: bool,
    /// The key details, present only when `valid` is `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<GetApiKeyResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetApiKeyExpirationRequest {
    /// The new expiration timestamp for the API key.
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetApiKeyMetadataRequest {
    /// The new metadata for the API key.
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FindApiKeysParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_ids: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_ids: Option<Vec<Uuid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespaces: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owners: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<Utc>>,
}

impl FindApiKeysParams {
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

    pub fn include_ids(mut self, include_ids: impl IntoIterator<Item = impl Into<Uuid>>) -> Self {
        self.include_ids = Some(include_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn exclude_ids(mut self, exclude_ids: impl IntoIterator<Item = impl Into<Uuid>>) -> Self {
        self.exclude_ids = Some(exclude_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn namespaces(mut self, namespaces: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.namespaces = Some(namespaces.into_iter().map(Into::into).collect());
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    pub fn short_keys(mut self, short_keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.short_keys = Some(short_keys.into_iter().map(Into::into).collect());
        self
    }

    pub fn owners(mut self, owners: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.owners = Some(owners.into_iter().map(Into::into).collect());
        self
    }

    pub fn revoked(mut self, revoked: bool) -> Self {
        self.revoked = Some(revoked);
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
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
