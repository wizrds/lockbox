pub mod traits;
pub mod set_api_key_last_used;

use async_trait::async_trait;
use std::any::Any;
use jobq::Task;

use crate::{service::errors::ApiKeyServiceError, tasks::traits::TaskOperation};


#[derive(Debug)]
pub enum TaskOperations {
    SetApiKeyLastUsed(set_api_key_last_used::SetApiKeyLastUsedTask)
}

#[async_trait]
impl Task for TaskOperations {
    type Output = Box<dyn Any + Send + Sync>;
    type Error = ApiKeyServiceError;

    async fn execute(&self) -> Result<Self::Output, Self::Error> {
        match self {
            TaskOperations::SetApiKeyLastUsed(task) => task
                .execute()
                .await
                .map(|result| Box::new(result) as Self::Output),
        }
    }
}