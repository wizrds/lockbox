use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{
    entities::{api_key, api_key_namespace, api_key_tag},
    service::errors::{ApiKeyServiceError, ApiKeyServiceResult},
    crypto::{generate_random_alphanumeric, get_checksum},
};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TagFilter {
    Value(String),
    Null,
    Unspecified,
}

impl Default for TagFilter {
    fn default() -> Self {
        TagFilter::Unspecified
    }
}

impl TagFilter {
    pub fn into_val(&self) -> Option<Option<String>> {
        match self {
            TagFilter::Value(v) => Some(Some(v.clone())),
            TagFilter::Null => Some(None),
            TagFilter::Unspecified => None,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateApiKeyPayload {
    pub owner: String,
    pub scope: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub tag: Option<String>,
    pub metadata: HashMap<String, String>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateApiKeyResponse {
    pub id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub owner: String,
    pub scope: Option<String>,
    pub tag: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl CreateApiKeyResponse {
    pub fn from_model(model: api_key::Model, key_parts: ApiKeyParts) -> Self {
        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            namespace: model.namespace,
            key: key_parts.token(),
            created_at: model.created_at,
            owner: model.owner,
            scope: model.scope,
            tag: model.tag,
            expires_at: model.expires_at,
            metadata: model.metadata.into(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetApiKeyResponse {
    pub id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub short_key: String,
    pub created_at: DateTime<Utc>,
    pub owner: String,
    pub scope: Option<String>,
    pub tag: Option<String>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl GetApiKeyResponse {
    pub fn from_model(model: api_key::Model) -> Self {
        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            namespace: model.namespace,
            short_key: model.short_key,
            created_at: model.created_at,
            owner: model.owner,
            scope: model.scope,
            tag: model.tag,
            revoked: model.revoked,
            revoked_at: model.revoked_at,
            expires_at: model.expires_at,
            last_used_at: model.last_used_at,
            metadata: model.metadata.into(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntrospectApiKeyResponse {
    pub valid: bool,
    pub key: Option<GetApiKeyResponse>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateNamespaceResponse {
    pub tenant_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub is_default: bool,
}

impl CreateNamespaceResponse {
    pub fn from_model(model: api_key_namespace::Model) -> Self {
        Self {
            tenant_id: model.tenant_id,
            name: model.name,
            created_at: model.created_at,
            is_default: model.is_default,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetNamespaceResponse {
    pub tenant_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub is_default: bool,
}

impl GetNamespaceResponse {
    pub fn from_model(model: api_key_namespace::Model) -> Self {
        Self {
            tenant_id: model.tenant_id,
            name: model.name,
            created_at: model.created_at,
            is_default: model.is_default,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTagResponse {
    pub tenant_id: String,
    pub namespace: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl CreateTagResponse {
    pub fn from_model(model: api_key_tag::Model) -> Self {
        Self {
            tenant_id: model.tenant_id,
            namespace: model.namespace,
            name: model.name,
            created_at: model.created_at,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GetTagResponse {
    pub tenant_id: String,
    pub namespace: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl GetTagResponse {
    pub fn from_model(model: api_key_tag::Model) -> Self {
        Self {
            tenant_id: model.tenant_id,
            namespace: model.namespace,
            name: model.name,
            created_at: model.created_at,
        }
    }
}


/// Represents the parts of an API key.
/// 
/// A full API key typically takes the following format:
/// `{namespace}_{tag}_{short_key}{long_key}{checksum}`
/// Or
/// `{namespace}_{short_key}{long_key}{checksum}`
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiKeyParts {
    pub namespace: String,
    pub tag: Option<String>,
    pub short_key: String,
    pub long_key: String,
}

impl ApiKeyParts {
    pub fn new(namespace: String, tag: Option<String>, short_key: String, long_key: String) -> Self {
        Self {
            namespace,
            tag,
            short_key,
            long_key,
        }
    }

    /// Generates a checksum for the API key parts.
    /// 
    /// # Returns
    /// A string representing the checksum.
    pub fn checksum(&self) -> String {
        get_checksum(&[
            &self.namespace,
            &self.tag.as_deref().unwrap_or_default(),
            &self.short_key,
            &self.long_key
        ])
        .chars()
        .take(6)
        .collect::<String>()
    }

    /// Constructs the full API key token from its parts.
    /// 
    /// # Returns
    /// A string representing the full API key token.
    pub fn token(&self) -> String {
        match &self.tag {
            Some(tag) => format!(
                "{}_{}_{}{}{}",
                self.namespace,
                tag,
                self.short_key,
                self.long_key,
                self.checksum()
            ),
            None => format!(
                "{}_{}{}{}",
                self.namespace,
                self.short_key,
                self.long_key,
                self.checksum()
            )
        }
    }

    /// Creates an `ApiKeyParts` instance from a token string.
    /// 
    /// # Arguments
    /// * `token` - A string slice representing the API key token.
    /// 
    /// # Returns
    /// A `ApiKeyServiceResult<Self>` which is either an `ApiKeyParts` instance or an error.
    /// 
    /// # Errors
    /// Returns an error if the token format is invalid or if parsing fails.
    pub fn from_token(token: &str) -> ApiKeyServiceResult<Self> {
        let parts = token.split('_').collect::<Vec<&str>>();

        let (namespace, tag, key_blob) = match parts.len() {
            2 => (
                parts[0].to_string(),
                None,
                parts[1].to_string(),
            ),
            3 => (
                parts[0].to_string(),
                Some(parts[1].to_string()),
                parts[2].to_string(),
            ),
            _ => return Err(ApiKeyServiceError::InvalidKey),
        };

        // short_key (12) + long_key (24) + checksum (6)
        if key_blob.len() != 42 {
            return Err(ApiKeyServiceError::InvalidKey);
        }

        let short_key = key_blob[..12].to_string();
        let long_key = key_blob[12..36].to_string();
        let checksum = key_blob[36..].to_string();

        let key_parts = Self::new(namespace, tag, short_key, long_key);

        if key_parts.checksum() != checksum {
            return Err(ApiKeyServiceError::InvalidKey);
        }

        Ok(key_parts)
    }

    /// Generates a new `ApiKeyParts` instance with the given namespace.
    /// 
    /// # Arguments
    /// * `namespace` - A string slice representing the namespace for the API key.
    /// 
    /// # Returns
    /// A new `ApiKeyParts` instance with the specified namespace and generated keys.
    pub fn generate(namespace: &str, tag: Option<&str>) -> Self {
        Self::new(
            namespace.to_string(),
            tag.map(|t| t.to_string()),
            generate_random_alphanumeric(12),
            generate_random_alphanumeric(24),
        )
    }
}