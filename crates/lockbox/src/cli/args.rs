use async_trait::async_trait;
use anyhow::Result;
use rustls::crypto::aws_lc_rs;
use clap::{Parser, Subcommand, CommandFactory};

use lockbox_core::telemetry::Telemetry;

use crate::{
    config::Config,
    cli::{commands::*, error::CliError, traits::Cmd, context::Ctx}
};


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
    /// Print the full command reference as markdown and exit.
    #[clap(long)]
    pub help_detail: bool,

    #[arg(long, short)]
    pub config_file: Option<String>,

    #[clap(subcommand)]
    pub command: Option<CliCommands>,
}

impl CliArgs {
    pub async fn update_ctx(&self, ctx: &mut Ctx) -> Result<()> {
        ctx.config_file = self.config_file.clone();
        ctx.app_config = Config::builder()
            .with_optional_file(ctx.config_file.clone().as_deref())
            .build()
            .await?;

        Ok(())
    }

    pub async fn run(telemetry: Telemetry) {
        let _ = aws_lc_rs::default_provider().install_default();
        let args = Self::parse();
        let mut ctx = Ctx::new(telemetry);

        match &args.command {
            Some(command) => {
                args
                    .update_ctx(&mut ctx)
                    .await
                    .map_err(CliError::from)
                    .unwrap_or_else(|e| e.exit());
                (command as &dyn Cmd)
                    .walk_execute(&mut ctx)
                    .await
                    .map_err(CliError::from)
                    .unwrap_or_else(|e| e.exit());
            },
            _ => {
                Self::command()
                    .print_help()
                    .map_err(CliError::from)
                    .unwrap_or_else(|e| e.exit());

                std::process::exit(1);
            }
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum CliCommands {
    #[clap(name = "completions", about = "Generate shell completions")]
    Completions(completions::CliCommandCompletions),
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
            CliCommands::Completions(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Serve(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Migrate(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Config(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Namespaces(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::Tags(cmd) => Some(cmd as &dyn Cmd),
            CliCommands::ApiKeys(cmd) => Some(cmd as &dyn Cmd),
        }
    }
}