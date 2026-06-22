use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{service::{api_key::{ApiKeyServiceTrait, ApiKeySvc}, errors::ApiKeyServiceError}, tasks::traits::TaskOperation};


#[derive(Debug)]
pub struct SetApiKeyLastUsedTask {
    pub api_key_service: Arc<ApiKeySvc>,
    pub id: Uuid,
    pub last_used_at: DateTime<Utc>,
    pub tenant_id: String,
}

#[async_trait]
impl TaskOperation for SetApiKeyLastUsedTask {
    type Output = ();
    type Error = ApiKeyServiceError;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        self.api_key_service
            .set_api_key_last_used(self.id, self.last_used_at, self.tenant_id.clone())
            .await
    }
}