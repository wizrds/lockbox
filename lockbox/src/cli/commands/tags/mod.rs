pub mod create;
pub mod get;
pub mod delete;
pub mod find;

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use clap::{Args, Subcommand};

use lockbox_core::{
    database::Database,
    service::api_key::ApiKeyService,
    repository::{
        api_key::ApiKeyRepository,
        api_key_namespace::ApiKeyNamespaceRepository,
        api_key_tag::ApiKeyTagRepository,
    },
};

use crate::cli::{traits::Cmd, context::{Ctx, CtxTags}};


#[derive(Clone, Args, Debug)]
pub struct CliCommandTags {
    #[arg(long, short, help = "The tenant ID")]
    pub tenant_id: Option<String>,
    #[arg(long, short, help = "The namespace")]
    pub namespace: Option<String>,

    #[command(subcommand)]
    pub command: Option<CliCommandTagsCommand>,
}

#[async_trait]
impl Cmd for CliCommandTags {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        let database = Arc::new(
            Database::builder()
                .with_primary(ctx.app_config.database.primary.clone())
                .with_options(ctx.app_config.database.options.clone())
                .with_migrations(false)
                .build()
                .await?
        );
        ctx.tags = Some(CtxTags {
            tenant_id: self.tenant_id.clone().unwrap_or(ctx.app_config.default_tenant_id.clone()),
            namespace: self.namespace.clone().unwrap_or(ctx.app_config.default_namespace.clone()),
            api_key_service: Some(
                ApiKeyService::builder()
                    .with_key_repo(ApiKeyRepository::new(database.clone()))
                    .with_namespace_repo(ApiKeyNamespaceRepository::new(database.clone()))
                    .with_tag_repo(ApiKeyTagRepository::new(database.clone()))
                    .build()
            ),
            ..Default::default()
        });

        Ok(())
    }
    
    fn next_cmd(&self) -> Option<&dyn Cmd> {
        self.command
            .as_ref()
            .map(|cmd| cmd as &dyn Cmd)
    }
}

#[derive(Clone, Subcommand, Debug)]
pub enum CliCommandTagsCommand {
    #[command(name = "create", about = "Create a new tag in a namespace")]
    Create(create::CliCommandTagsCreate),
    #[command(name = "get", about = "Get a tag by name in a namespace")]
    Get(get::CliCommandTagsGet),
    #[command(name = "delete", about = "Delete a tag by name in a namespace")]
    Delete(delete::CliCommandTagsDelete),
    #[command(name = "find", about = "Find tags")]
    Find(find::CliCommandTagsFind),
}

#[async_trait]
impl Cmd for CliCommandTagsCommand {
    fn next_cmd(&self) -> Option<&dyn Cmd> {
        match self {
            CliCommandTagsCommand::Create(cmd) => Some(cmd as &dyn Cmd),
            CliCommandTagsCommand::Get(cmd) => Some(cmd as &dyn Cmd),
            CliCommandTagsCommand::Delete(cmd) => Some(cmd as &dyn Cmd),
            CliCommandTagsCommand::Find(cmd) => Some(cmd as &dyn Cmd),
        }
    }
}