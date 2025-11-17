use async_trait::async_trait;
use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{config::AppConfig, cli::{commands::*, traits::Cmd, context::Ctx}};


#[derive(Parser, Debug)]
#[
    clap(
        name = "lockbox",
        version,
        author = "Tim Pogue",
        about = "Lightweight, efficient, API Key authentication service"
    )
]
pub struct CliArgs {
    #[arg(long, short)]
    pub config_file: Option<String>,

    #[clap(subcommand)]
    pub command: Option<CliCommands>,
}

impl CliArgs {
    pub async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.config_file = self.config_file.clone();
        ctx.app_config = AppConfig::builder()
            .with_env()
            .with_optional_file(ctx.config_file.clone().as_deref())
            .build()?;

        Ok(())
    }
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    #[clap(name = "serve", about = "Start the lockbox server")]
    Serve(serve::CliCommandServe),
    #[clap(name = "migrate", about = "Run database migrations")]
    Migrate(migrate::CliCommandMigrate),
    #[clap(name = "config", about = "View resolved configuration")]
    Config(config::CliCommandConfig),
    #[clap(name = "namespaces", about = "Manage namespaces")]
    Namespaces(namespaces::CliCommandNamespaces),
    #[clap(name = "tags", about = "Manage tags")]
    Tags(tags::CliCommandTags),
    #[clap(name = "api-keys", about = "Manage API Keys")]
    ApiKeys(api_keys::CliCommandApiKeys),
}

#[async_trait]
impl Cmd for CliCommands {
    fn next_cmd(&self) -> Option<&dyn Cmd> {
        match self {
            CliCommands::Serve(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Migrate(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Config(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Namespaces(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Tags(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::ApiKeys(cmd) => Some(cmd as &dyn Cmd),
        }
    }
}