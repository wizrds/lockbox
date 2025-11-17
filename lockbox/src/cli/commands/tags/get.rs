use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::{Args, ArgAction};
use validator::Validate;

use lockbox_core::service::{api_key::ApiKeyServiceTrait, errors::ApiKeyServiceError};

use crate::{cli::{context::Ctx, traits::Cmd}, manifest::{to_serialized_string_pretty, ManifestFormat}};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandTagsGet {
    #[arg(help = "The Tag name")]
    #[validate(length(max = 6))]
    pub name: String,
    #[arg(long, action = ArgAction::SetTrue, help = "Skip if the tag does not exist")]
    pub skip_missing: bool,
    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandTagsGet {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err));
        }

        let tags_ctx = ctx.tags.as_mut().unwrap();

        tags_ctx.tag_name = Some(self.name.clone());
        tags_ctx.skip_missing = self.skip_missing;
        tags_ctx.format = Some(self.format.clone());

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let tags_ctx = ctx.tags.as_ref().unwrap();

        match tags_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .get_tag(
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
            Ok(tag) => {
                println!("{}", to_serialized_string_pretty(&tag, tags_ctx.format.clone().unwrap())?);
                Ok(())
            },
            Err(ApiKeyServiceError::TagNotFound) if tags_ctx.skip_missing => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}
