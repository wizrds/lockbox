use async_trait::async_trait;
use std::fmt::{Debug, Display};


#[async_trait]
pub trait TaskOperation {
    type Output: Send + Sync;
    type Error: Debug + Display + Send + Sync + 'static;

    async fn execute(&self) -> Result<Self::Output, Self::Error>;
}