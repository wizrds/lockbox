use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use sea_orm::{FromJsonQueryResult, entity::prelude::*};
use chrono::{DateTime, Utc};
use uuid::Uuid;


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct Metadata(HashMap<String, String>);

impl Metadata {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn as_inner(&self) -> &HashMap<String, String> {
        &self.0
    }

    pub fn as_mut_inner(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }

    pub fn into_inner(self) -> HashMap<String, String> {
        self.0
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    pub fn extend(&mut self, entries: impl IntoIterator<Item = (String, String)>) {
        self.0.extend(entries);
    }

    pub fn get(&self, key: impl Into<String>) -> Option<&String> {
        self.0.get(&key.into())
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, String>> for Metadata {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

impl From<Metadata> for HashMap<String, String> {
    fn from(metadata: Metadata) -> Self {
        metadata.0
    }
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "api_key")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub tag: Option<String>,
    pub short_key: String,
    pub long_key_hash: String,
    pub owner: String,
    pub scope: Option<String>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub metadata: Metadata,
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}
