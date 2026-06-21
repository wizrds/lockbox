use async_trait::async_trait;
use anyhow::Result;
use clap::Args;

use crate::{cli::{context::{Ctx, CtxConfig}, traits::Cmd}, manifest::{to_serialized_string_pretty, ManifestFormat}};


#[derive(Clone, Args, Debug)]
pub struct CliCommandConfig {
    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandConfig {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.config = Some(CtxConfig {
            format: Some(self.format.clone())
        });

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let config_ctx = ctx.config.as_ref().unwrap();

        println!("{}", to_serialized_string_pretty(&ctx.app_config, config_ctx.format.clone().unwrap())?);

        Ok(())
    }
}