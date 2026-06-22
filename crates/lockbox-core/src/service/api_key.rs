use std::collections::{HashSet, HashMap};
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub use crate::repository::{
    api_key::FindApiKeyParams,
    api_key_namespace::FindApiKeyNamespaceParams,
    api_key_tag::FindApiKeyTagParams,
};

use crate::{
    database::{orm::ActiveValue, paginate::Page},
    service::{
        errors::{ApiKeyServiceError, ApiKeyServiceResult},
        types::{
            CreateApiKeyPayload,
            CreateApiKeyResponse,
            GetApiKeyResponse,
            IntrospectApiKeyResponse,
            ApiKeyParts,
            CreateNamespaceResponse,
            GetNamespaceResponse,
            CreateTagResponse,
            GetTagResponse,
        },
    },
    repository::{
        api_key::{ApiKeyRepository, ApiKeyRepositoryTrait},
        api_key_namespace::{ApiKeyNamespaceRepository, ApiKeyNamespaceRepositoryTrait},
        api_key_tag::{ApiKeyTagRepository, ApiKeyTagRepositoryTrait},
    },
    crypto::{hash_sha512, constant_cmp},
    entities::{api_key, api_key_namespace, api_key_tag},
};


#[async_trait]
pub trait ApiKeyServiceTrait {
    /// Create a new API key.
    /// 
    /// # Arguments
    /// * `payload`: The payload containing the API key details.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the [`CreateApiKeyResponse`](crate::service::types::CreateApiKeyResponse)
    async fn create_api_key(&self, payload: CreateApiKeyPayload, tenant_id: String) -> ApiKeyServiceResult<CreateApiKeyResponse>;
    /// Introspect an API key to validate it and retrieve its details.
    /// 
    /// # Arguments
    /// * `token`: The API key token.
    /// * `scope`: An optional scope to restrict the introspection.
    /// * `tags`: The expected tags. Use `None` to allow any, `Some(vec![])` to require none,
    ///           and `Some(vec!["tag1", "tag2"])` to require one of the specific tags. If None is provided as one
    ///           of those items then it allows no tags as well.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the [`IntrospectApiKeyResponse`](crate::service::types::IntrospectApiKeyResponse)
    async fn introspect_api_key(&self, token: String, scope: Option<String>, tags: Option<Vec<Option<String>>>, tenant_id: String) -> ApiKeyServiceResult<IntrospectApiKeyResponse>;
    /// Revoke an API key.
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to revoke.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn revoke_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Rotate an API key.
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to rotate.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the new [`CreateApiKeyResponse`](crate::service::types::CreateApiKeyResponse) derived from
    /// the rotated key.
    async fn rotate_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<CreateApiKeyResponse>;
    /// Set the last used at on an API key
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to update.
    /// * `last_used_at`: The new last used timestamp.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn set_api_key_last_used(&self, id: Uuid, last_used_at: DateTime<Utc>, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Set the expiration date on an API key
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to update.
    /// * `expiration`: The new expiration date.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn set_api_key_expiration(&self, id: Uuid, expiration: DateTime<Utc>, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Set the metadata on an API key
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to update.
    /// * `metadata`: The new metadata to set.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn set_api_key_metadata(&self, id: Uuid, metadata: HashMap<String, String>, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Delete an API key.
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to delete.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn delete_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Get an API key.
    /// 
    /// # Arguments
    /// * `id`: The ID of the API key to retrieve.
    /// * `tenant_id`: The tenant ID of the API key.
    /// 
    /// # Returns
    /// A Result containing the [`GetApiKeyResponse`](crate::service::types::GetApiKeyResponse)
    async fn get_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<GetApiKeyResponse>;
    /// Find API keys.
    /// 
    /// # Arguments
    /// * `params`: The parameters to filter the API keys.
    /// 
    /// # Returns
    /// A Result containing a page of [`GetApiKeyResponse`](crate::service::types::GetApiKeyResponse)
    async fn find_api_keys(&self, params: FindApiKeyParams) -> ApiKeyServiceResult<Page<GetApiKeyResponse>>;
    /// Create a new namespace.
    /// 
    /// # Arguments
    /// * `name`: The name of the namespace.
    /// * `tenant_id`: The tenant ID of the namespace.
    /// 
    /// # Returns
    /// A Result containing the [`CreateNamespaceResponse`](crate::service::types::CreateNamespaceResponse)
    async fn create_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<CreateNamespaceResponse>;
    /// Get a namespace.
    /// 
    /// # Arguments
    /// * `name`: The name of the namespace.
    /// * `tenant_id`: The tenant ID of the namespace.
    /// 
    /// # Returns
    /// A Result containing the [`GetNamespaceResponse`](crate::service::types::GetNamespaceResponse)
    async fn get_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<GetNamespaceResponse>;
    /// Get the default namespace for a tenant.
    /// 
    /// # Arguments
    /// * `tenant_id`: The tenant ID of the namespace.
    /// 
    /// # Returns
    /// A Result containing the [`GetNamespaceResponse`](crate::service::types::GetNamespaceResponse)
    async fn get_default_namespace(&self, tenant_id: String) -> ApiKeyServiceResult<GetNamespaceResponse>;
    /// Set the default namespace for a tenant.
    /// 
    /// When the namespace specified is set to the default, all other namespaces for the tenant
    /// are set to not default ensuring a tenant only has a single default namespace.
    /// 
    /// # Arguments
    /// * `name`: The name of the namespace.
    /// * `tenant_id`: The tenant ID of the namespace.
    /// 
    /// # Returns
    /// A Result containing the [`GetNamespaceResponse`](crate::service::types::GetNamespaceResponse)
    async fn set_default_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<GetNamespaceResponse>;
    /// Delete a namespace.
    /// 
    /// # Arguments
    /// * `name`: The name of the namespace.
    /// * `tenant_id`: The tenant ID of the namespace.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn delete_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Find namespaces.
    /// 
    /// # Arguments
    /// * `params`: The parameters to filter the namespaces.
    /// 
    /// # Returns
    /// A Result containing a page of [`GetNamespaceResponse`](crate::service::types::GetNamespaceResponse)
    async fn find_namespaces(&self, params: FindApiKeyNamespaceParams) -> ApiKeyServiceResult<Page<GetNamespaceResponse>>;
    /// Create a new tag.
    /// 
    /// # Arguments
    /// * `name`: The name of the tag.
    /// * `namespace`: The namespace of the tag.
    /// * `tenant_id`: The tenant ID of the tag.
    /// 
    /// # Returns
    /// A Result containing the [`CreateTagResponse`](crate::service::types::CreateTagResponse)
    async fn create_tag(&self, name: String, namespace: String, tenant_id: String) -> ApiKeyServiceResult<CreateTagResponse>;
    /// Get a tag.
    /// 
    /// # Arguments
    /// * `name`: The name of the tag.
    /// * `namespace`: The namespace of the tag.
    /// * `tenant_id`: The tenant ID of the tag.
    /// 
    /// # Returns
    /// A Result containing the [`GetTagResponse`](crate::service::types::GetTagResponse)
    async fn get_tag(&self, name: String, namespace: String, tenant_id: String) -> ApiKeyServiceResult<GetTagResponse>;
    /// Delete a tag.
    /// 
    /// # Arguments
    /// * `name`: The name of the tag.
    /// * `namespace`: The namespace of the tag.
    /// * `tenant_id`: The tenant ID of the tag.
    /// 
    /// # Returns
    /// A Result containing the success status of the operation.
    async fn delete_tag(&self, name: String, namespace: String, tenant_id: String) -> ApiKeyServiceResult<()>;
    /// Find tags.
    /// 
    /// # Arguments
    /// * `params`: The parameters to filter the tags.
    /// 
    /// # Returns
    /// A Result containing a page of [`GetTagResponse`](crate::service::types::GetTagResponse)
    async fn find_tags(&self, params: FindApiKeyTagParams) -> ApiKeyServiceResult<Page<GetTagResponse>>;
}


#[derive(Debug)]
pub struct ApiKeyService<K, N, T>
where 
    K: ApiKeyRepositoryTrait + Send + Sync,
    N: ApiKeyNamespaceRepositoryTrait + Send + Sync,
    T: ApiKeyTagRepositoryTrait + Send + Sync,
{
    key_repo: K,
    namespace_repo: N,
    tag_repo: T,
}


impl<K, N, T> ApiKeyService<K, N, T>
where
    K: ApiKeyRepositoryTrait + Send + Sync,
    N: ApiKeyNamespaceRepositoryTrait + Send + Sync,
    T: ApiKeyTagRepositoryTrait + Send + Sync,
{
    pub fn new(key_repo: K, namespace_repo: N, tag_repo: T) -> Self {
        Self {
            key_repo,
            namespace_repo,
            tag_repo,
        }
    }

    pub fn builder() -> ApiKeyServiceBuilder<K, N, T> {
        ApiKeyServiceBuilder::new()
    }
}


#[async_trait]
impl<K, N, T> ApiKeyServiceTrait for ApiKeyService<K, N, T>
where
    K: ApiKeyRepositoryTrait + Send + Sync,
    N: ApiKeyNamespaceRepositoryTrait + Send + Sync,
    T: ApiKeyTagRepositoryTrait + Send + Sync,
{
    async fn create_api_key(&self, payload: CreateApiKeyPayload, tenant_id: String) -> ApiKeyServiceResult<CreateApiKeyResponse> {
        let namespace = self.namespace_repo
            .get_default_namespace(tenant_id.clone())
            .await?
            .ok_or(ApiKeyServiceError::NamespaceNotFound)?;

        let namespace_tag = match payload.tag {
            Some(t) => match self.tag_repo.get_tag((t, namespace.name.clone(), tenant_id.clone())).await? {
                Some(tag) => Some(tag),
                None => return Err(ApiKeyServiceError::TagNotFound),
            },
            None => None,
        };

        let key_parts = ApiKeyParts::generate(&namespace.name, namespace_tag.as_ref().map(|t| t.name.as_str()));
        let api_key = self.key_repo
            .insert_api_keys(vec![
                api_key::ActiveModel {
                    id: ActiveValue::set(Uuid::new_v4()),
                    tenant_id: ActiveValue::set(tenant_id),
                    namespace: ActiveValue::set(key_parts.namespace.clone()),
                    tag: match namespace_tag {
                        Some(ns_tag) => ActiveValue::set(Some(ns_tag.name)),
                        None => ActiveValue::not_set(),
                    },
                    short_key: ActiveValue::set(key_parts.short_key.clone()),
                    long_key_hash: ActiveValue::set(hash_sha512(&key_parts.long_key)),
                    owner: ActiveValue::set(payload.owner),
                    scope: ActiveValue::set(payload.scope),
                    revoked: ActiveValue::set(false),
                    expires_at: match payload.expires_at {
                        Some(exp) => ActiveValue::set(Some(exp)),
                        None => ActiveValue::not_set(),
                    },
                    metadata: ActiveValue::set(payload.metadata.into()),
                    created_at: ActiveValue::set(Utc::now()),
                    ..Default::default()
                }
            ])
            .await?
            .into_iter()
            .next()
            .unwrap();

        Ok(CreateApiKeyResponse::from_model(api_key, key_parts))
    }

    async fn introspect_api_key(
        &self,
        token: String,
        scope: Option<String>,
        tags: Option<Vec<Option<String>>>,
        tenant_id: String
    ) -> ApiKeyServiceResult<IntrospectApiKeyResponse> {
        let parts = ApiKeyParts::from_token(&token)?;

        if !match tags {
            None => true,
            Some(ref tag_list) if tag_list.is_empty() => parts.tag.is_none(),
            Some(ref tag_list) => tag_list.contains(&parts.tag),
        } {
            return Err(ApiKeyServiceError::InvalidKey);
        }

        let mut find_params = FindApiKeyParams::new(1, 1)
            .with_tenant_ids(vec![tenant_id])
            .with_namespaces(vec![parts.namespace])
            .with_short_keys(vec![parts.short_key]);

        if let Some(tag_part) = parts.tag {
            find_params = find_params.with_tags(vec![tag_part]);
        }

        let api_keys = self.key_repo
            .find_api_keys(find_params)
            .await?;

        if api_keys.items.is_empty() {
            return Err(ApiKeyServiceError::InvalidKey);
        }

        let api_key = api_keys
            .items
            .into_iter()
            .next()
            .unwrap();

        if !constant_cmp(&hash_sha512(&parts.long_key), &api_key.long_key_hash) {
            return Err(ApiKeyServiceError::InvalidKey);
        }

        if api_key.revoked {
            return Err(ApiKeyServiceError::InvalidKey);
        }

        if api_key.expires_at.is_some() && api_key.expires_at.unwrap() < Utc::now() {
            return Err(ApiKeyServiceError::KeyExpired);
        }

        if let Some(required_scope) = scope {
            let available_scopes = api_key.scope
                .as_deref()
                .unwrap_or("")
                .split_whitespace()
                .collect::<HashSet<_>>();

            if !required_scope.split_whitespace().all(|s| available_scopes.contains(s)) {
                return Err(ApiKeyServiceError::InsufficientScope);
            }
        }

        Ok(IntrospectApiKeyResponse {
            valid: true,
            key: Some(GetApiKeyResponse::from_model(api_key)),
        })
    }

    async fn revoke_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.key_repo.get_api_key(id, Some(tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::KeyNotFound);
        }

        self.key_repo
            .update_api_keys(vec![api_key::ActiveModel {
                id: ActiveValue::set(id),
                revoked: ActiveValue::set(true),
                revoked_at: ActiveValue::set(Some(chrono::Utc::now())),
                ..Default::default()
            }])
            .await?;

        Ok(())
    }

    async fn rotate_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<CreateApiKeyResponse> {
        let api_key = match self.key_repo.get_api_key(id, Some(tenant_id.clone())).await? {
            Some(key) => key,
            None => return Err(ApiKeyServiceError::KeyNotFound),
        };

        let new_parts = ApiKeyParts::generate(&api_key.namespace, api_key.tag.as_deref());
        let new_key = self.key_repo
            .insert_api_keys(vec![
                api_key::ActiveModel {
                    id: ActiveValue::set(api_key.id),
                    tenant_id: ActiveValue::set(api_key.tenant_id),
                    namespace: ActiveValue::set(new_parts.namespace.clone()),
                    tag: match new_parts.tag {
                        Some(ref tag) => ActiveValue::set(Some(tag.clone())),
                        None => ActiveValue::not_set(),
                    },
                    short_key: ActiveValue::set(new_parts.short_key.clone()),
                    long_key_hash: ActiveValue::set(hash_sha512(&new_parts.long_key)),
                    owner: ActiveValue::set(api_key.owner),
                    scope: ActiveValue::set(api_key.scope),
                    ..Default::default()
                }
            ])
            .await?
            .into_iter()
            .next()
            .unwrap();

        self.key_repo
            .update_api_keys(vec![api_key::ActiveModel {
                id: ActiveValue::set(api_key.id),
                revoked: ActiveValue::set(true),
                revoked_at: ActiveValue::set(Some(chrono::Utc::now())),
                ..Default::default()
            }])
            .await?;

        Ok(CreateApiKeyResponse::from_model(new_key, new_parts))
    }

    async fn set_api_key_last_used(&self, id: Uuid, last_used_at: DateTime<Utc>, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.key_repo.get_api_key(id, Some(tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::KeyNotFound);
        }

        self.key_repo
            .update_api_keys(vec![api_key::ActiveModel {
                id: ActiveValue::set(id),
                last_used_at: ActiveValue::set(Some(last_used_at)),
                ..Default::default()
            }])
            .await?;

        Ok(())
    }

    async fn set_api_key_expiration(&self, id: Uuid, expiration: DateTime<Utc>, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.key_repo.get_api_key(id, Some(tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::KeyNotFound);
        }

        self.key_repo
            .update_api_keys(vec![api_key::ActiveModel {
                id: ActiveValue::set(id),
                expires_at: ActiveValue::set(Some(expiration)),
                ..Default::default()
            }])
            .await?;

        Ok(())
    }

    async fn set_api_key_metadata(&self, id: Uuid, metadata: HashMap<String, String>, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.key_repo.get_api_key(id, Some(tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::KeyNotFound);
        }

        self.key_repo
            .update_api_keys(vec![api_key::ActiveModel {
                id: ActiveValue::set(id),
                metadata: ActiveValue::set(metadata.into()),
                ..Default::default()
            }])
            .await?;

        Ok(())
    }

    async fn delete_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.key_repo.get_api_key(id, Some(tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::KeyNotFound);
        }

        self.key_repo.delete_api_keys(vec![id], Some(tenant_id.clone())).await?;

        Ok(())
    }

    async fn get_api_key(&self, id: Uuid, tenant_id: String) -> ApiKeyServiceResult<GetApiKeyResponse> {
        let api_key = match self.key_repo.get_api_key(id, Some(tenant_id.clone())).await? {
            Some(key) => key,
            None => return Err(ApiKeyServiceError::KeyNotFound),
        };

        Ok(GetApiKeyResponse::from_model(api_key))
    }

    async fn find_api_keys(&self, params: FindApiKeyParams) -> ApiKeyServiceResult<Page<GetApiKeyResponse>> {
        let api_keys = self.key_repo.find_api_keys(params).await?;

        Ok(Page {
            items: api_keys.items
                .into_iter()
                .map(GetApiKeyResponse::from_model)
                .collect(),
            count: api_keys.count,
            next_page: api_keys.next_page,
            previous_page: api_keys.previous_page,
        })
    }

    async fn create_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<CreateNamespaceResponse> {
        if self.namespace_repo.get_namespace((name.clone(), tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::NamespaceAlreadyExists);
        }

        let namespace = self.namespace_repo
            .insert_namespaces(vec![api_key_namespace::ActiveModel {
                name: ActiveValue::set(name.clone()),
                tenant_id: ActiveValue::set(tenant_id.clone()),
                is_default: match self.namespace_repo
                    .get_default_namespace(tenant_id.clone())
                    .await?
                    .is_some() {
                    true => ActiveValue::not_set(),
                    false => ActiveValue::set(true),
                },
                ..Default::default()
            }])
            .await?
            .into_iter()
            .next()
            .unwrap();

        Ok(CreateNamespaceResponse::from_model(namespace))
    }

    async fn get_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<GetNamespaceResponse> {
        let namespace = match self.namespace_repo.get_namespace((name, tenant_id)).await? {
            Some(ns) => ns,
            None => return Err(ApiKeyServiceError::NamespaceNotFound),
        };

        Ok(GetNamespaceResponse::from_model(namespace))
    }

    async fn get_default_namespace(&self, tenant_id: String) -> ApiKeyServiceResult<GetNamespaceResponse> {
        let namespace = match self.namespace_repo.get_default_namespace(tenant_id).await? {
            Some(ns) => ns,
            None => return Err(ApiKeyServiceError::NamespaceNotFound),
        };

        Ok(GetNamespaceResponse::from_model(namespace))
    }

    async fn set_default_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<GetNamespaceResponse> {
        if !self.namespace_repo.get_namespace((name.clone(), tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::NamespaceNotFound);
        }

        self.namespace_repo
            .set_default_namespace((name.clone(), tenant_id.clone()))
            .await?;

        let namespace = self.namespace_repo
            .get_namespace((name, tenant_id))
            .await?
            .expect("Namespace should exist after being set as default");

        Ok(GetNamespaceResponse::from_model(namespace))
    }

    async fn delete_namespace(&self, name: String, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.namespace_repo.get_namespace((name.clone(), tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::NamespaceNotFound);
        }

        self.namespace_repo
            .delete_namespaces(vec![(name, tenant_id)])
            .await?;

        Ok(())
    }

    async fn find_namespaces(&self, params: FindApiKeyNamespaceParams) -> ApiKeyServiceResult<Page<GetNamespaceResponse>> {
        let namespaces = self.namespace_repo.find_namespaces(params).await?;

        Ok(Page {
            items: namespaces.items
                .into_iter()
                .map(GetNamespaceResponse::from_model)
                .collect(),
            count: namespaces.count,
            next_page: namespaces.next_page,
            previous_page: namespaces.previous_page,
        })
    }

    async fn create_tag(&self, name: String, namespace: String, tenant_id: String) -> ApiKeyServiceResult<CreateTagResponse> {
        if self.tag_repo.get_tag((name.clone(), namespace.clone(), tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::TagAlreadyExists);
        }
        if self.namespace_repo.get_namespace((namespace.clone(), tenant_id.clone())).await?.is_none() {
            return Err(ApiKeyServiceError::NamespaceNotFound);
        }

        let tag = self.tag_repo
            .insert_tags(vec![api_key_tag::ActiveModel {
                tenant_id: ActiveValue::set(tenant_id),
                namespace: ActiveValue::set(namespace),
                name: ActiveValue::set(name),
                created_at: ActiveValue::set(chrono::Utc::now()),
                ..Default::default()
            }])
            .await?
            .into_iter()
            .next()
            .unwrap();

        Ok(CreateTagResponse::from_model(tag))
    }

    async fn get_tag(&self, name: String, namespace: String, tenant_id: String) -> ApiKeyServiceResult<GetTagResponse> {
        let tag = match self.tag_repo.get_tag((name, namespace, tenant_id)).await? {
            Some(t) => t,
            None => return Err(ApiKeyServiceError::TagNotFound),
        };

        Ok(GetTagResponse::from_model(tag))
    }

    async fn delete_tag(&self, name: String, namespace: String, tenant_id: String) -> ApiKeyServiceResult<()> {
        if !self.tag_repo.get_tag((name.clone(), namespace.clone(), tenant_id.clone())).await?.is_some() {
            return Err(ApiKeyServiceError::TagNotFound);
        }

        self.tag_repo
            .delete_tags(vec![(name, namespace, tenant_id)])
            .await?;

        Ok(())
    }

    async fn find_tags(&self, params: FindApiKeyTagParams) -> ApiKeyServiceResult<Page<GetTagResponse>> {
        let tags = self.tag_repo.find_tags(params).await?;

        Ok(Page {
            items: tags.items
                .into_iter()
                .map(GetTagResponse::from_model)
                .collect(),
            count: tags.count,
            next_page: tags.next_page,
            previous_page: tags.previous_page,
        })
    }
}

/// A type alias for the ApiKeyService implementation with the DB repository implementations
pub type ApiKeySvc = ApiKeyService<ApiKeyRepository, ApiKeyNamespaceRepository, ApiKeyTagRepository>;


pub struct ApiKeyServiceBuilder<K, N, T>
where
    K: ApiKeyRepositoryTrait + Send + Sync,
    N: ApiKeyNamespaceRepositoryTrait + Send + Sync,
    T: ApiKeyTagRepositoryTrait + Send + Sync,
{
    key_repo: Option<K>,
    namespace_repo: Option<N>,
    tag_repo: Option<T>,
}

impl<K, N, T> ApiKeyServiceBuilder<K, N, T>
where
    K: ApiKeyRepositoryTrait + Send + Sync,
    N: ApiKeyNamespaceRepositoryTrait + Send + Sync,
    T: ApiKeyTagRepositoryTrait + Send + Sync,
{
    pub fn new() -> Self {
        Self {
            key_repo: None,
            namespace_repo: None,
            tag_repo: None,
        }
    }

    pub fn with_key_repo(mut self, repo: K) -> Self {
        self.key_repo = Some(repo);
        self
    }

    pub fn with_namespace_repo(mut self, repo: N) -> Self {
        self.namespace_repo = Some(repo);
        self
    }

    pub fn with_tag_repo(mut self, repo: T) -> Self {
        self.tag_repo = Some(repo);
        self
    }

    pub fn build(self) -> ApiKeyService<K, N, T> {
        ApiKeyService {
            key_repo: self.key_repo.expect("Key repository must be set"),
            namespace_repo: self.namespace_repo.expect("Namespace repository must be set"),
            tag_repo: self.tag_repo.expect("Tag repository must be set"),
        }
    }
}