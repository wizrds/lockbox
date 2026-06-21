use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::{Args, ArgAction};
use validator::Validate;

use lockbox_core::service::{api_key::ApiKeyServiceTrait, errors::ApiKeyServiceError};

use crate::cli::{context::Ctx, traits::Cmd};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandNamespacesDelete {
    #[arg(help = "The Namespace name")]
    #[validate(length(max = 6))]
    pub name: String,
    #[arg(long, action = ArgAction::SetTrue, help = "Skip if the namespace does not exist")]
    pub skip_missing: bool,
}

#[async_trait]
impl Cmd for CliCommandNamespacesDelete {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err));
        }

        let namespaces_ctx = ctx.namespaces.as_mut().unwrap();

        namespaces_ctx.namespace_name = Some(self.name.clone());
        namespaces_ctx.skip_missing = self.skip_missing;

        Ok(())
    }

    async fn run(&self, ctx: &mut Ctx) -> Result<()> {
        let namespaces_ctx = ctx.namespaces.as_ref().unwrap();

        match namespaces_ctx
            .api_key_service
            .as_ref()
            .unwrap()
            .delete_namespace(
                namespaces_ctx
                    .namespace_name
                    .clone()
                    .unwrap(),
                namespaces_ctx
                    .tenant_id
                    .clone(),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(ApiKeyServiceError::NamespaceNotFound) if namespaces_ctx.skip_missing => Ok(()),
            Err(err) => return Err(err.into()),
        }
    }
}