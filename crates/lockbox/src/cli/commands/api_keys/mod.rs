pub mod create;
pub mod get;
pub mod delete;
pub mod find;
pub mod introspect;
pub mod revoke;
pub mod rotate;
pub mod set_expiration;
pub mod set_metadata;

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use validator::Validate;

use lockbox_core::{
    database::Database,
    service::api_key::ApiKeyService,
    repository::{
        api_key::ApiKeyRepository,
        api_key_namespace::ApiKeyNamespaceRepository,
        api_key_tag::ApiKeyTagRepository,
    },
};

use crate::cli::{traits::Cmd, context::{Ctx, CtxApiKeys}};


#[derive(Clone, Args, Debug, Validate)]
pub struct CliCommandApiKeys {
    #[arg(long, short, help = "The tenant ID")]
    pub tenant_id: Option<String>,
    #[arg(long, short, help = "The namespace")]
    #[validate(length(max = 6))]
    pub namespace: Option<String>,

    #[command(subcommand)]
    pub command: Option<CliCommandApiKeysCommand>,
}

#[async_trait]
impl Cmd for CliCommandApiKeys {
    async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        if let Err(err) = self.validate() {
            return Err(anyhow!("Validation error: {}", err));
        }

        let database = Arc::new(
            Database::builder()
                .with_primary(ctx.app_config.database.primary.clone())
                .with_options(ctx.app_config.database.options.clone())
                .with_migrations(false)
                .build()
                .await?
        );
        ctx.api_keys = Some(CtxApiKeys {
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
pub enum CliCommandApiKeysCommand {
    #[command(name = "create", about = "Create a new API key")]
    Create(create::CliCommandApiKeysCreate),
    #[command(name = "get", about = "Get an API key by ID")]
    Get(get::CliCommandApiKeysGet),
    #[command(name = "delete", about = "Delete an API key by ID")]
    Delete(delete::CliCommandApiKeysDelete),
    #[command(name = "find", about = "Find API keys")]
    Find(find::CliCommandApiKeysFind),
    #[command(name = "introspect", about = "Introspect an API key")]
    Introspect(introspect::CliCommandApiKeysIntrospect),
    #[command(name = "revoke", about = "Revoke an API key")]
    Revoke(revoke::CliCommandApiKeysRevoke),
    #[command(name = "rotate", about = "Rotate an API key")]
    Rotate(rotate::CliCommandApiKeysRotate),
    #[command(name = "set-expiration", about = "Set the expiration date for an API key")]
    SetExpiration(set_expiration::CliCommandApiKeysSetExpiration),
    #[command(name = "set-metadata", about = "Set the metadata for an API key")]
    SetMetadata(set_metadata::CliCommandApiKeysSetMetadata),
}

#[async_trait]
impl Cmd for CliCommandApiKeysCommand {
    fn next_cmd(&self) -> Option<&dyn Cmd> {
        match self {
            CliCommandApiKeysCommand::Create(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::Get(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::Delete(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::Find(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::Introspect(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::Revoke(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::Rotate(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::SetExpiration(cmd) => Some(cmd as &dyn Cmd),
            CliCommandApiKeysCommand::SetMetadata(cmd) => Some(cmd as &dyn Cmd),
        }
    }
}