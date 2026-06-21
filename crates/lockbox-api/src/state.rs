use std::sync::Arc;
use jobq::{JobQueue, FifoQueue, Job};

use lockbox_core::{
    database::Database,
    service::api_key::ApiKeyService,
    repository::{
        api_key::ApiKeyRepository,
        api_key_namespace::ApiKeyNamespaceRepository,
        api_key_tag::ApiKeyTagRepository,
    },
    tasks::TaskOperations,
};

pub struct ApiState {
    pub default_tenant_id: String,
    pub default_namespace: String,
    pub job_queue: Arc<JobQueue<TaskOperations, FifoQueue<Job<TaskOperations>>>>,
    pub database: Arc<Database>,
    pub api_key_service: Arc<ApiKeyService<ApiKeyRepository, ApiKeyNamespaceRepository, ApiKeyTagRepository>>,
}


impl ApiState {
    pub fn builder() -> ApiStateBuilder {
        ApiStateBuilder::new()
    }
}

pub struct ApiStateBuilder {
    default_tenant_id: Option<String>,
    default_namespace: Option<String>,
    job_queue: Option<Arc<JobQueue<TaskOperations, FifoQueue<Job<TaskOperations>>>>>,
    database: Option<Arc<Database>>,
    api_key_service: Option<Arc<ApiKeyService<ApiKeyRepository, ApiKeyNamespaceRepository, ApiKeyTagRepository>>>,
}

impl ApiStateBuilder {
    pub fn new() -> Self {
        Self {
            default_tenant_id: None,
            default_namespace: None,
            job_queue: None,
            database: None,
            api_key_service: None,
        }
    }

    pub fn with_default_tenant_id(mut self, id: impl Into<String>) -> Self {
        self.default_tenant_id = Some(id.into());
        self
    }

    pub fn with_default_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.default_namespace = Some(namespace.into());
        self
    }

    pub fn with_job_queue(self, job_queue: JobQueue<TaskOperations, FifoQueue<Job<TaskOperations>>>) -> Self {
        self.with_job_queue_arc(Arc::new(job_queue))
    }

    pub fn with_job_queue_arc(mut self, job_queue: Arc<JobQueue<TaskOperations, FifoQueue<Job<TaskOperations>>>>) -> Self {
        self.job_queue = Some(job_queue);
        self
    }

    pub fn with_database(self, database: Database) -> Self {
        self.with_database_arc(Arc::new(database))
    }

    pub fn with_database_arc(mut self, database: Arc<Database>) -> Self {
        self.database = Some(database);
        self
    }

    pub fn with_api_key_service(self, service: ApiKeyService<ApiKeyRepository, ApiKeyNamespaceRepository, ApiKeyTagRepository>) -> Self {
        self.with_api_key_service_arc(Arc::new(service))
    }

    pub fn with_api_key_service_arc(mut self, service: Arc<ApiKeyService<ApiKeyRepository, ApiKeyNamespaceRepository, ApiKeyTagRepository>>) -> Self {
        self.api_key_service = Some(service);
        self
    }

    pub fn build(self) -> ApiState {
        ApiState {
            default_tenant_id: self.default_tenant_id.expect("Missing default tenant id"),
            default_namespace: self.default_namespace.expect("Missing default namespace"),
            job_queue: self.job_queue.expect("Missing job queue"),
            database: self.database.expect("Missing database"),
            api_key_service: self.api_key_service.expect("Missing API key service"),
        }
    }
}
