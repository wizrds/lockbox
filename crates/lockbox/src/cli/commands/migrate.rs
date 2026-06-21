use async_trait::async_trait;
use anyhow::Result;
use clap::Args;

use lockbox_core::database::Database;

use crate::cli::{traits::Cmd, context::{Ctx, CtxMigrate}};


#[derive(Clone, Args, Debug)]
pub struct CliCommandMigrate {}

#[async_trait]
impl Cmd for CliCommandMigrate {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.migrate = Some(CtxMigrate {});

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        Database::builder()
            .with_primary(ctx.app_config.database.primary.clone())
            .with_options(ctx.app_config.database.options.clone())
            .with_migrations(true)
            .build()
            .await?;

        Ok(())
    }
}