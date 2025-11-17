use std::{sync::Arc, collections::HashMap};
use async_trait::async_trait;
use uuid::Uuid;
use futures::{TryFutureExt, future::try_join_all};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    database::{
        database::Database,
        orm::{
            ConnectionTrait,
            ActiveModelTrait,
            EntityTrait,
            ColumnTrait,
            QueryFilter,
            Condition,
        },
        ext::JsonColumnExt,
        paginate::{Page, paginate},
        errors::{DatabaseError, DatabaseResult},
    },
    entities::api_key,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindApiKeyParams {
    pub page: u64,
    pub per_page: u64,
    pub include_ids: Option<Vec<Uuid>>,
    pub exclude_ids: Option<Vec<Uuid>>,
    pub tenant_ids: Option<Vec<String>>,
    pub namespaces: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub short_keys: Option<Vec<String>>,
    pub owners: Option<Vec<String>>,
    pub revoked: Option<bool>,
    pub metadata: Option<HashMap<String, String>>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

impl FindApiKeyParams {
    pub fn new(page: u64, per_page: u64) -> Self {
        Self {
            page,
            per_page,
            include_ids: None,
            exclude_ids: None,
            tenant_ids: None,
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

    pub fn with_include_ids(mut self, ids: Vec<impl Into<Uuid>>) -> Self {
        self.include_ids = Some(ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_exclude_ids(mut self, ids: Vec<impl Into<Uuid>>) -> Self {
        self.exclude_ids = Some(ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_tenant_ids(mut self, tenant_ids: Vec<impl Into<String>>) -> Self {
        self.tenant_ids = Some(tenant_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_namespaces(mut self, namespaces: Vec<impl Into<String>>) -> Self {
        self.namespaces = Some(namespaces.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = Some(tags.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_short_keys(mut self, short_keys: Vec<impl Into<String>>) -> Self {
        self.short_keys = Some(short_keys.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_owners(mut self, owners: Vec<impl Into<String>>) -> Self {
        self.owners = Some(owners.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_revoked(mut self, revoked: bool) -> Self {
        self.revoked = Some(revoked);
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_created_before(mut self, created_before: DateTime<Utc>) -> Self {
        self.created_before = Some(created_before);
        self
    }

    pub fn with_created_after(mut self, created_after: DateTime<Utc>) -> Self {
        self.created_after = Some(created_after);
        self
    }
}


#[async_trait]
pub trait ApiKeyRepositoryTrait {
    /// Insert new API keys
    /// 
    /// # Arguments
    /// * `payloads` - The new API keys to insert
    /// 
    /// # Returns
    /// A vector of the inserted API keys
    async fn insert_api_keys(&self, payloads: Vec<api_key::ActiveModel>) -> DatabaseResult<Vec<api_key::Model>>;
    /// Update existing API keys
    /// 
    /// # Arguments
    /// * `payloads` - The API keys to update
    /// 
    /// # Returns
    /// A vector of the updated API keys
    async fn update_api_keys(&self, payloads: Vec<api_key::ActiveModel>) -> DatabaseResult<Vec<api_key::Model>>;
    /// Set the last used timestamp for an API key
    /// 
    /// # Arguments
    /// * `id` - The ID of the API key to update
    /// * `last_used` - The new last used timestamp
    /// * `tenant_id` - The ID of the tenant that owns the API key
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn set_last_used(&self, id: Uuid, last_used: DateTime<Utc>, tenant_id: Option<String>) -> DatabaseResult<()>;
    /// Get an API key by its ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the API key to retrieve
    /// * `tenant_id` - The ID of the tenant that owns the API key
    ///
    /// # Returns
    /// The requested API key, if found, otherwise None
    async fn get_api_key(&self, id: Uuid, tenant_id: Option<String>) -> DatabaseResult<Option<api_key::Model>>;
    /// Delete API keys by their IDs
    /// 
    /// # Arguments
    /// * `ids` - The IDs of the API keys to delete
    /// * `tenant_id` - The ID of the tenant that owns the API keys
    /// 
    /// # Returns
    /// A result indicating success or failure
    async fn delete_api_keys(&self, ids: Vec<Uuid>, tenant_id: Option<String>) -> DatabaseResult<()>;
    /// Find API keys based on the given filters
    /// 
    /// # Arguments
    /// * `params` - The filters to apply when searching for API keys
    /// 
    /// # Returns
    /// A page of API keys matching the filters
    async fn find_api_keys(&self, params: FindApiKeyParams) -> DatabaseResult<Page<api_key::Model>>;
}


#[derive(Debug)]
pub struct ApiKeyRepository {
    database: Arc<Database>,
}

impl ApiKeyRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl ApiKeyRepositoryTrait for ApiKeyRepository {
    async fn insert_api_keys(&self, payloads: Vec<api_key::ActiveModel>) -> DatabaseResult<Vec<api_key::Model>> {
        if payloads.is_empty() {
            return Ok(vec![]);
        }
        
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key::Entity::insert_many(payloads)
                        .exec_with_returning_many(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn update_api_keys(&self, payloads: Vec<api_key::ActiveModel>) -> DatabaseResult<Vec<api_key::Model>> {
        if payloads.is_empty() {
            return Ok(vec![]);
        }

        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    try_join_all(
                        payloads
                            .into_iter()
                            .map(|payload| payload.update(txn).map_err(DatabaseError::from))
                            .collect::<Vec<_>>()
                    )
                    .await
                })
            })
            .await
    }

    async fn set_last_used(&self, id: Uuid, last_used: DateTime<Utc>, tenant_id: Option<String>) -> DatabaseResult<()> {
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key::Entity::update_many()
                        .col_expr(api_key::Column::LastUsedAt, last_used.into())
                        .filter(
                            Condition::all()
                                .add(api_key::Column::Id.eq(id))
                                .add_option(tenant_id.map(|t| api_key::Column::TenantId.eq(t)))
                        )
                        .exec(txn)
                        .await
                        .map_err(DatabaseError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn get_api_key(&self, id: Uuid, tenant_id: Option<String>) -> DatabaseResult<Option<api_key::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    api_key::Entity::find()
                        .filter(
                            Condition::all()
                                .add(api_key::Column::Id.eq(id))
                                .add_option(tenant_id.map(|t| api_key::Column::TenantId.eq(t)))
                        )
                        .one(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn delete_api_keys(&self, ids: Vec<Uuid>, tenant_id: Option<String>) -> DatabaseResult<()> {
        if ids.is_empty() {
            return Ok(());
        }
        
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key::Entity::delete_many()
                        .filter(
                            Condition::all()
                                .add(api_key::Column::Id.is_in(ids))
                                .add_option(tenant_id.map(|t| api_key::Column::TenantId.eq(t)))
                        )
                        .exec(txn)
                        .await
                        .map_err(DatabaseError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn find_api_keys(&self, params: FindApiKeyParams) -> DatabaseResult<Page<api_key::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    let mut query = api_key::Entity::find();

                    if let Some(ids) = params.include_ids {
                        if !ids.is_empty() {
                            query = query.filter(api_key::Column::Id.is_in(ids));
                        }
                    }
                    if let Some(ids) = params.exclude_ids {
                        if !ids.is_empty() {
                            query = query.filter(api_key::Column::Id.is_not_in(ids));
                        }
                    }
                    if let Some(tenant_ids) = params.tenant_ids {
                        if !tenant_ids.is_empty() {
                            query = query.filter(api_key::Column::TenantId.is_in(tenant_ids));
                        }
                    }
                    if let Some(namespaces) = params.namespaces {
                        if !namespaces.is_empty() {
                            query = query.filter(api_key::Column::Namespace.is_in(namespaces));
                        }
                    }
                    if let Some(short_keys) = params.short_keys {
                        if !short_keys.is_empty() {
                            query = query.filter(api_key::Column::ShortKey.is_in(short_keys));
                        }
                    }
                    if let Some(owners) = params.owners {
                        if !owners.is_empty() {
                            query = query.filter(api_key::Column::Owner.is_in(owners));
                        }
                    }
                    if let Some(revoked) = params.revoked {
                        query = query.filter(api_key::Column::Revoked.eq(revoked));
                    }
                    if let Some(created_before) = params.created_before {
                        query = query.filter(api_key::Column::CreatedAt.lt(created_before));
                    }
                    if let Some(created_after) = params.created_after {
                        query = query.filter(api_key::Column::CreatedAt.gt(created_after));
                    }
                    if let Some(metadata) = params.metadata {
                        query = query.filter(
                            metadata
                                .into_iter()
                                .fold(Condition::all(), |cond, (k, v)| {
                                    cond.add(api_key::Column::Metadata.json_key_eq(&txn.get_database_backend(), k, v))
                                })
                        );
                    }

                    paginate(txn, query, params.page, params.per_page)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }
}