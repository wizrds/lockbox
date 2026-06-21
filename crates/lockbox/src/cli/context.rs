use uuid::Uuid;
use chrono::{DateTime, Utc};

use lockbox_core::service::api_key::ApiKeySvc;

use crate::{config::AppConfig, manifest::ManifestFormat};


#[derive(Default, Debug)]
pub struct CtxTaskQueue {
    pub worker_count: Option<usize>,
    pub max_queue_capacity: Option<usize>,
    pub batch_size: Option<usize>,
    pub batch_timeout_ms: Option<usize>,
}


#[derive(Default, Debug)]
pub struct CtxServe {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub task_queue: CtxTaskQueue,
    pub migrations: bool,
}

#[derive(Default, Debug)]
pub struct CtxMigrate {}

#[derive(Default, Debug)]
pub struct CtxConfig {
    pub format: Option<ManifestFormat>,
}

#[derive(Default, Debug)]
pub struct CtxNamespaces {
    // Global
    pub tenant_id: String,
    pub format: Option<ManifestFormat>,
    pub api_key_service: Option<ApiKeySvc>,

    // Create | Get | Delete
    pub namespace_name: Option<String>,
    pub skip_exists: bool,
    pub skip_missing: bool,

    // Find
    pub page: u64,
    pub per_page: u64,
    pub tenant_ids: Option<Vec<String>>,
    pub names: Option<Vec<String>>,
    pub is_default: Option<bool>,
}

#[derive(Default, Debug)]
pub struct CtxTags {
    pub tenant_id: String,
    pub namespace: String,
    pub format: Option<ManifestFormat>,
    pub api_key_service: Option<ApiKeySvc>,

    // Create | Get | Delete
    pub tag_name: Option<String>,
    pub skip_exists: bool,
    pub skip_missing: bool,

    // Find
    pub page: u64,
    pub per_page: u64,
    pub tenant_ids: Option<Vec<String>>,
    pub namespaces: Option<Vec<String>>,
    pub names: Option<Vec<String>>,
}

#[derive(Default, Debug)]
pub struct CtxApiKeys {
    pub tenant_id: String,
    pub format: Option<ManifestFormat>,
    pub api_key_service: Option<ApiKeySvc>,

    // Create
    pub owner: Option<String>,
    pub expiration: Option<DateTime<Utc>>,
    pub metadata: Option<Vec<(String, String)>>,
    pub skip_exists: bool,

    // Get | Delete
    pub id: Option<Uuid>,
    pub skip_missing: bool,

    // Introspect
    pub api_key: Option<String>,
    pub required_tags: Option<Vec<Option<String>>>,
    pub no_tag: bool,

    // Create
    pub scope: Option<String>,
    pub tag: Option<String>,

    // Find
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
}

#[derive(Default, Debug)]
pub struct Ctx {
    pub config_file: Option<String>,
    pub app_config: AppConfig,
    pub serve: Option<CtxServe>,
    pub migrate: Option<CtxMigrate>,
    pub config: Option<CtxConfig>,
    pub namespaces: Option<CtxNamespaces>,
    pub tags: Option<CtxTags>,
    pub api_keys: Option<CtxApiKeys>,
}