use std::sync::Arc;
use async_trait::async_trait;
use futures::{TryFutureExt, future::try_join_all};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    database::{
        database::Database,
        orm::{
            Condition,
            ActiveModelTrait,
            EntityTrait,
            ColumnTrait,
            QueryFilter,
        },
        paginate::{Page, paginate},
        errors::{DatabaseError, DatabaseResult},
    },
    entities::api_key_namespace,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindApiKeyNamespaceParams {
    pub page: u64,
    pub per_page: u64,
    pub tenant_ids: Option<Vec<String>>,
    pub names: Option<Vec<String>>,
    pub is_default: Option<bool>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

impl FindApiKeyNamespaceParams {
    pub fn new(page: u64, per_page: u64) -> Self {
        Self {
            page,
            per_page,
            tenant_ids: None,
            names: None,
            is_default: None,
            created_before: None,
            created_after: None,
        }
    }

    pub fn with_tenant_ids(mut self, tenant_ids: Vec<impl Into<String>>) -> Self {
        self.tenant_ids = Some(tenant_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_names(mut self, names: Vec<impl Into<String>>) -> Self {
        self.names = Some(names.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_is_default(mut self, is_default: bool) -> Self {
        self.is_default = Some(is_default);
        self
    }

    pub fn with_created_before(mut self, created_before: impl Into<DateTime<Utc>>) -> Self {
        self.created_before = Some(created_before.into());
        self
    }

    pub fn with_created_after(mut self, created_after: impl Into<DateTime<Utc>>) -> Self {
        self.created_after = Some(created_after.into());
        self
    }
}


#[async_trait]
pub trait ApiKeyNamespaceRepositoryTrait {
    /// Insert new API key namespaces
    /// 
    /// # Arguments
    /// * `payloads` - The new namespaces to insert
    /// 
    /// # Returns
    /// A vector of the inserted API key namespaces
    async fn insert_namespaces(&self, payloads: Vec<api_key_namespace::ActiveModel>) -> DatabaseResult<Vec<api_key_namespace::Model>>;
    /// Update existing API key namespaces
    /// 
    /// # Arguments
    /// * `payloads` - The namespaces to update
    /// 
    /// # Returns
    /// A vector of the updated API key namespaces
    async fn update_namespaces(&self, payloads: Vec<api_key_namespace::ActiveModel>) -> DatabaseResult<Vec<api_key_namespace::Model>>;
    /// Get an API key namespace by its ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the namespace to retrieve, (name, tenant_id)
    /// 
    /// # Returns
    /// The requested API key namespace, if found, otherwise None
    async fn get_namespace(&self, id: (String, String)) -> DatabaseResult<Option<api_key_namespace::Model>>;
    /// Get the default namespace for a given Tenant ID
    /// 
    /// # Arguments
    /// * `tenant_id` - The ID of the tenant to retrieve the default namespace for
    ///
    /// # Returns
    /// The default API key namespace for the given tenant, if found, otherwise None
    async fn get_default_namespace(&self, tenant_id: String) -> DatabaseResult<Option<api_key_namespace::Model>>;
    /// Set the default API key namespace for a given Tenant ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the namespace to set as default, (name, tenant_id)
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn set_default_namespace(&self, id: (String, String)) -> DatabaseResult<()>;
    /// Delete API key namespaces by their IDs
    ///
    /// # Arguments
    /// * `ids` - The IDs of the namespaces to delete, (name, tenant_id)
    ///
    /// # Returns
    /// A result indicating success or failure
    async fn delete_namespaces(&self, ids: Vec<(String, String)>) -> DatabaseResult<()>;
    /// Find API key namespaces based on the given filters
    ///
    /// # Arguments
    /// * `params` - The filters to apply when searching for namespaces
    ///
    /// # Returns
    /// A paginated list of API key namespaces matching the filters
    async fn find_namespaces(&self, params: FindApiKeyNamespaceParams) -> DatabaseResult<Page<api_key_namespace::Model>>;
}


#[derive(Debug)]
pub struct ApiKeyNamespaceRepository {
    database: Arc<Database>,
}

impl ApiKeyNamespaceRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl ApiKeyNamespaceRepositoryTrait for ApiKeyNamespaceRepository {
    async fn insert_namespaces(&self, payloads: Vec<api_key_namespace::ActiveModel>) -> DatabaseResult<Vec<api_key_namespace::Model>> {
        if payloads.is_empty() {
            return Ok(vec![]);
        }
        
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key_namespace::Entity::insert_many(payloads)
                        .exec_with_returning_many(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn update_namespaces(&self, payloads: Vec<api_key_namespace::ActiveModel>) -> DatabaseResult<Vec<api_key_namespace::Model>> {
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

    async fn get_namespace(&self, id: (String, String)) -> DatabaseResult<Option<api_key_namespace::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    api_key_namespace::Entity::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn get_default_namespace(&self, tenant_id: String) -> DatabaseResult<Option<api_key_namespace::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    api_key_namespace::Entity::find()
                        .filter(api_key_namespace::Column::TenantId.eq(tenant_id))
                        .filter(api_key_namespace::Column::IsDefault.eq(true))
                        .one(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn set_default_namespace(&self, id: (String, String)) -> DatabaseResult<()> {
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key_namespace::Entity::update_many()
                        .col_expr(api_key_namespace::Column::IsDefault, true.into())
                        .filter(api_key_namespace::Column::Name.eq(id.0.clone()))
                        .filter(api_key_namespace::Column::TenantId.eq(id.1.clone()))
                        .exec(txn)
                        .await
                        .map_err(DatabaseError::from)?;

                    api_key_namespace::Entity::update_many()
                        .col_expr(api_key_namespace::Column::IsDefault, false.into())
                        .filter(api_key_namespace::Column::Name.ne(id.0.clone()))
                        .filter(api_key_namespace::Column::TenantId.eq(id.1.clone()))
                        .exec(txn)
                        .await
                        .map_err(DatabaseError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn delete_namespaces(&self, ids: Vec<(String, String)>) -> DatabaseResult<()> {
        if ids.is_empty() {
            return Ok(());
        }
        
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key_namespace::Entity::delete_many()
                        .filter(
                            ids
                                .into_iter()
                                .fold(Condition::any(), |cond, (name, tenant_id)| {
                                    cond.add(
                                        Condition::all()
                                            .add(api_key_namespace::Column::Name.eq(name))
                                            .add(api_key_namespace::Column::TenantId.eq(tenant_id))
                                    )
                                })
                        )
                        .exec(txn)
                        .await
                        .map_err(DatabaseError::from)?;

                    Ok(())
                })
            })
            .await
    }

    async fn find_namespaces(&self, params: FindApiKeyNamespaceParams) -> DatabaseResult<Page<api_key_namespace::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    let mut query = api_key_namespace::Entity::find();

                    if let Some(names) = params.names {
                        if !names.is_empty() {
                            query = query.filter(api_key_namespace::Column::Name.is_in(names));
                        }
                    }
                    if let Some(tenant_ids) = params.tenant_ids {
                        if !tenant_ids.is_empty() {
                            query = query.filter(api_key_namespace::Column::TenantId.is_in(tenant_ids));
                        }
                    }
                    if let Some(is_default) = params.is_default {
                        query = query.filter(api_key_namespace::Column::IsDefault.eq(is_default));
                    }
                    if let Some(created_before) = params.created_before {
                        query = query.filter(api_key_namespace::Column::CreatedAt.lt(created_before));
                    }
                    if let Some(created_after) = params.created_after {
                        query = query.filter(api_key_namespace::Column::CreatedAt.gt(created_after));
                    }

                    paginate(txn, query, params.page, params.per_page)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }
}