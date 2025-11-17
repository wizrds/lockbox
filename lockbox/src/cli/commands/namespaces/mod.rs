pub mod create;
pub mod get;
pub mod delete;
pub mod find;
pub mod set_default;
pub mod get_default;

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

use crate::cli::{traits::Cmd, context::{Ctx, CtxNamespaces}};


#[derive(Clone, Args, Debug)]
pub struct CliCommandNamespaces {
    #[arg(long, short, help = "The tenant ID")]
    pub tenant_id: Option<String>,

    #[command(subcommand)]
    pub command: Option<CliCommandNamespacesCommand>,
}

#[async_trait]
impl Cmd for CliCommandNamespaces {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        let database = Arc::new(
            Database::builder()
                .with_primary(ctx.app_config.database.primary.clone())
                .with_options(ctx.app_config.database.options.clone())
                .with_migrations(false)
                .build()
                .await?
        );
        ctx.namespaces = Some(CtxNamespaces {
            tenant_id: self.tenant_id.clone().unwrap_or(ctx.app_config.default_tenant_id.clone()),
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
pub enum CliCommandNamespacesCommand {
    #[command(name = "create", about = "Create a new namespace")]
    Create(create::CliCommandNamespacesCreate),
    #[command(name = "get", about = "Get a namespace")]
    Get(get::CliCommandNamespacesGet),
    #[command(name = "delete", about = "Delete a namespace")]
    Delete(delete::CliCommandNamespacesDelete),
    #[command(name = "find", about = "Find namespaces")]
    Find(find::CliCommandNamespacesFind),
    #[command(name = "set-default", about = "Set a namespace as the default for a tenant")]
    SetDefault(set_default::CliCommandNamespacesSetDefault),
    #[command(name = "get-default", about = "Get the default namespace for a tenant")]
    GetDefault(get_default::CliCommandNamespacesGetDefault),
}

#[async_trait]
impl Cmd for CliCommandNamespacesCommand {
    fn next_cmd(&self) -> Option<&dyn Cmd> {
        match self {
            CliCommandNamespacesCommand::Create(cmd) => Some(cmd as &dyn Cmd),
            CliCommandNamespacesCommand::Get(cmd) => Some(cmd as &dyn Cmd),
            CliCommandNamespacesCommand::Delete(cmd) => Some(cmd as &dyn Cmd),
            CliCommandNamespacesCommand::Find(cmd) => Some(cmd as &dyn Cmd),
            CliCommandNamespacesCommand::SetDefault(cmd) => Some(cmd as &dyn Cmd),
            CliCommandNamespacesCommand::GetDefault(cmd) => Some(cmd as &dyn Cmd),
        }
    }
}