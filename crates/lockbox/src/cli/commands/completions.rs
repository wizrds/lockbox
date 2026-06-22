use std::io;

use async_trait::async_trait;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};

use crate::cli::{args::CliArgs, context::Ctx, traits::Cmd};


#[derive(Debug, Clone, Args)]
pub struct CliCommandCompletions {
    /// The shell to generate a completion script for.
    pub shell: Shell,
}

#[async_trait]
impl Cmd for CliCommandCompletions {
    async fn run(&self, _ctx: &mut Ctx) -> Result<(), anyhow::Error> {
        generate(self.shell, &mut CliArgs::command(), "lockbox", &mut io::stdout());
        Ok(())
    }
}
