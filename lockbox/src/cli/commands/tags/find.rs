use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::Args;
use validator::Validate;

use lockbox_core::service::api_key::{ApiKeyServiceTrait, FindApiKeyTagParams};

use crate::{
    cli::{context::Ctx, traits::Cmd, utils:: validate_vec_string_len},
    manifest::{to_serialized_string_pretty, ManifestFormat},
};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandTagsFind {
    #[arg(long, help = "The page number", default_value = "1")]
    pub page: u64,
    #[arg(long, help = "The number of items per page", default_value = "10")]
    pub per_page: u64,
    #[arg(long, help = "The tenant IDs to filter")]
    pub tenant_ids: Option<Vec<String>>,
    #[arg(long, help = "The namespaces to filter")]
    #[validate(custom(function = validate_vec_string_len::<6>))]
    pub namespaces: Option<Vec<String>>,
    #[arg(long, help = "The tag names to filter")]
    #[validate(custom(function = validate_vec_string_len::<6>))]
    pub names: Option<Vec<String>>,
    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandTagsFind {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err))
        }

        let tags_ctx = ctx.tags.as_mut().unwrap();

        tags_ctx.page = self.page;
        tags_ctx.per_page = self.per_page;
        tags_ctx.tenant_ids = self.tenant_ids.clone();
        tags_ctx.namespaces = self.namespaces.clone();
        tags_ctx.names = self.names.clone();
        tags_ctx.format = Some(self.format.clone());

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let tags_ctx = ctx.tags.as_ref().unwrap();

        let tags = tags_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .find_tags(
                FindApiKeyTagParams {
                    page: tags_ctx.page,
                    per_page: tags_ctx.per_page,
                    tenant_ids: tags_ctx.tenant_ids.clone(),
                    namespaces: tags_ctx.namespaces.clone(),
                    names: tags_ctx.names.clone(),
                    created_after: None,
                    created_before: None,
                },
            )
            .await?;

        println!("{}", to_serialized_string_pretty(&tags, tags_ctx.format.clone().unwrap())?);

        Ok(())
    }
}