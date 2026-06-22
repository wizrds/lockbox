use std::{future::Future, pin::Pin};
use async_trait::async_trait;
use anyhow::Result;

use crate::cli::context::Ctx;


#[async_trait]
pub trait Cmd: Send + Sync {
    async fn update_ctx(&self, _ctx: &mut Ctx) -> Result<()> {
        Ok(())
    }

    async fn run(&self, _ctx: &mut Ctx) -> Result<()> {
        Ok(())
    }

    fn next_cmd(&self) -> Option<&dyn Cmd> {
        None
    }
}

impl<'ctx> dyn Cmd + 'ctx {
    pub fn walk_execute(
        &'ctx self,
        ctx: &'ctx mut Ctx
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'ctx>> {
        Box::pin(async move {
            self.update_ctx(ctx).await?;
            self.run(ctx).await?;

            if let Some(next) = self.next_cmd() {
                next.walk_execute(ctx).await?;
            }

            Ok(())
        })
    }
}