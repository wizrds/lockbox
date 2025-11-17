use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::{Args, ArgAction};
use validator::Validate;

use lockbox_core::service::api_key::{ApiKeyServiceTrait, FindApiKeyNamespaceParams};

use crate::{
    cli::{context::Ctx, traits::Cmd, utils:: validate_vec_string_len},
    manifest::{to_serialized_string_pretty, ManifestFormat},
};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandNamespacesFind {
    #[arg(long, help = "The page number", default_value = "1")]
    pub page: u64,
    #[arg(long, help = "The number of items per page", default_value = "10")]
    pub per_page: u64,
    #[arg(long, help = "The tenant IDs to filter")]
    pub tenant_ids: Option<Vec<String>>,
    #[arg(long, help = "The namespace names to filter")]
    #[validate(custom(function = validate_vec_string_len::<6>))]
    pub names: Option<Vec<String>>,
    #[arg(long, action = ArgAction::Set, help = "Filter by default namespaces")]
    pub is_default: Option<bool>,
    #[arg(long, short, help = "The serialization format", default_value = "json")]
    pub format: ManifestFormat,
}

#[async_trait]
impl Cmd for CliCommandNamespacesFind {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err))
        }

        let namespaces_ctx = ctx.namespaces.as_mut().unwrap();

        namespaces_ctx.page = self.page;
        namespaces_ctx.per_page = self.per_page;
        namespaces_ctx.tenant_ids = self.tenant_ids.clone();
        namespaces_ctx.names = self.names.clone();
        namespaces_ctx.is_default = self.is_default;
        namespaces_ctx.format = Some(self.format.clone());

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let namespaces_ctx = ctx.namespaces.as_ref().unwrap();

        let namespaces = namespaces_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .find_namespaces(
                FindApiKeyNamespaceParams {
                    page: namespaces_ctx.page,
                    per_page: namespaces_ctx.per_page,
                    tenant_ids: Some(namespaces_ctx.tenant_ids
                        .clone()
                        .unwrap_or(vec![namespaces_ctx.tenant_id.clone()])
                    ),
                    names: namespaces_ctx.names.clone(),
                    is_default: namespaces_ctx.is_default,
                    created_after: None,
                    created_before: None,
                },
            )
            .await?;

        println!("{}", to_serialized_string_pretty(&namespaces, namespaces_ctx.format.clone().unwrap())?);

        Ok(())
    }
}