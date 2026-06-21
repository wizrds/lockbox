use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::{Args, ArgAction};
use validator::Validate;

use lockbox_core::service::{api_key::ApiKeyServiceTrait, errors::ApiKeyServiceError};

use crate::cli::{context::Ctx, traits::Cmd};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandTagsDelete {
    #[arg(help = "The Tag name")]
    #[validate(length(max = 6))]
    pub name: String,
    #[arg(long, action = ArgAction::SetTrue, help = "Skip if the tag does not exist")]
    pub skip_missing: bool,
}

#[async_trait]
impl Cmd for CliCommandTagsDelete {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err));
        }

        let tags_ctx = ctx.tags.as_mut().unwrap();

        tags_ctx.tag_name = Some(self.name.clone());
        tags_ctx.skip_missing = self.skip_missing;

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let tags_ctx = ctx.tags.as_ref().unwrap();

        match tags_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .delete_tag(
                tags_ctx
                    .tag_name
                    .clone()
                    .unwrap(),
                tags_ctx
                    .namespace
                    .clone(),
                tags_ctx
                    .tenant_id
                    .clone(),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(ApiKeyServiceError::TagNotFound) if tags_ctx.skip_missing => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}
