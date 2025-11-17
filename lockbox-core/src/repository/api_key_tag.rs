use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::{
    database::{
        database::Database,
        orm::{
            EntityTrait,
            ColumnTrait,
            QueryFilter,
            Condition,
        },
        paginate::{Page, paginate},
        errors::{DatabaseError, DatabaseResult},
    },
    entities::api_key_tag,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindApiKeyTagParams {
    pub page: u64,
    pub per_page: u64,
    pub tenant_ids: Option<Vec<String>>,
    pub namespaces: Option<Vec<String>>,
    pub names: Option<Vec<String>>,
    pub created_before: Option<DateTime<Utc>>,
    pub created_after: Option<DateTime<Utc>>,
}

impl FindApiKeyTagParams {
    pub fn new(page: u64, per_page: u64) -> Self {
        Self {
            page,
            per_page,
            tenant_ids: None,
            namespaces: None,
            names: None,
            created_before: None,
            created_after: None,
        }
    }

    pub fn with_tenant_ids(mut self, tenant_ids: Vec<impl Into<String>>) -> Self {
        self.tenant_ids = Some(tenant_ids.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_namespaces(mut self, namespaces: Vec<impl Into<String>>) -> Self {
        self.namespaces = Some(namespaces.into_iter().map(Into::into).collect());
        self
    }

    pub fn with_names(mut self, names: Vec<impl Into<String>>) -> Self {
        self.names = Some(names.into_iter().map(Into::into).collect());
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
pub trait ApiKeyTagRepositoryTrait {
    /// Insert new API key tags
    /// 
    /// # Arguments
    /// * `payloads` - The new tags to insert
    /// 
    /// # Returns
    /// A vector of the inserted API key tags
    async fn insert_tags(&self, payloads: Vec<api_key_tag::ActiveModel>) -> DatabaseResult<Vec<api_key_tag::Model>>;
    /// Get an API key tag by its ID
    /// 
    /// # Arguments
    /// * `id` - The ID of the tag to retrieve, (name, namespace, tenant_id)
    /// 
    /// # Returns
    /// The requested API key tag, if found, otherwise None
    async fn get_tag(&self, id: (String, String, String)) -> DatabaseResult<Option<api_key_tag::Model>>;
    /// Delete API key tags by their IDs
    /// 
    /// # Arguments
    /// * `ids` - The IDs of the tags to delete, (name, namespace, tenant_id)
    /// 
    /// # Returns
    /// A result indicating the success or failure of the operation
    async fn delete_tags(&self, ids: Vec<(String, String, String)>) -> DatabaseResult<()>;
    /// Find API key tags based on the given filters
    /// 
    /// # Arguments
    /// * `params` - The filters to apply when searching for tags
    /// 
    /// # Returns
    /// A paginated list of API key tags matching the filters
    async fn find_tags(&self, params: FindApiKeyTagParams) -> DatabaseResult<Page<api_key_tag::Model>>;
}


#[derive(Debug)]
pub struct ApiKeyTagRepository {
    database: Arc<Database>,
}

impl ApiKeyTagRepository {
    pub fn new(database: Arc<Database>) -> Self {
        Self { database }
    }
}

#[async_trait]
impl ApiKeyTagRepositoryTrait for ApiKeyTagRepository {
    async fn insert_tags(&self, payloads: Vec<api_key_tag::ActiveModel>) -> DatabaseResult<Vec<api_key_tag::Model>> {
        if payloads.is_empty() {
            return Ok(vec![]);
        }
        
        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key_tag::Entity::insert_many(payloads)
                        .exec_with_returning_many(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn get_tag(&self, id: (String, String, String)) -> DatabaseResult<Option<api_key_tag::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    api_key_tag::Entity::find_by_id(id)
                        .one(txn)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }

    async fn delete_tags(&self, ids: Vec<(String, String, String)>) -> DatabaseResult<()> {
        if ids.is_empty() {
            return Ok(());
        }

        self.database
            .rw_transaction(|txn| {
                Box::pin(async move {
                    api_key_tag::Entity::delete_many()
                        .filter(
                            ids
                                .into_iter()
                                .fold(Condition::any(), |cond, (name, namespace, tenant_id)| {
                                    cond.add(
                                        Condition::all()
                                            .add(api_key_tag::Column::Name.eq(name))
                                            .add(api_key_tag::Column::Namespace.eq(namespace))
                                            .add(api_key_tag::Column::TenantId.eq(tenant_id))
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

    async fn find_tags(&self, params: FindApiKeyTagParams) -> DatabaseResult<Page<api_key_tag::Model>> {
        self.database
            .ro_transaction(|txn| {
                Box::pin(async move {
                    let mut query = api_key_tag::Entity::find();

                    if let Some(tenant_ids) = params.tenant_ids {
                        if !tenant_ids.is_empty() {
                            query = query.filter(api_key_tag::Column::TenantId.is_in(tenant_ids));
                        }
                    }
                    if let Some(namespaces) = params.namespaces {
                        if !namespaces.is_empty() {
                            query = query.filter(api_key_tag::Column::Namespace.is_in(namespaces));
                        }
                    }
                    if let Some(names) = params.names {
                        if !names.is_empty() {
                            query = query.filter(api_key_tag::Column::Name.is_in(names));
                        }
                    }
                    if let Some(created_before) = params.created_before {
                        query = query.filter(api_key_tag::Column::CreatedAt.lt(created_before));
                    }
                    if let Some(created_after) = params.created_after {
                        query = query.filter(api_key_tag::Column::CreatedAt.gt(created_after));
                    }

                    paginate(txn, query, params.page, params.per_page)
                        .await
                        .map_err(DatabaseError::from)
                })
            })
            .await
    }
}