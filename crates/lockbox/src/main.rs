mod cli;
mod manifest;
mod constants;
mod config;

use std::process::exit;
use clap::{Parser, CommandFactory};
use rustls::crypto::aws_lc_rs;

use crate::cli::{CliArgs, Cmd, Ctx, CliError};


#[cfg(all(feature = "mimalloc", not(feature = "stdalloc")))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
    let _ = aws_lc_rs::default_provider().install_default();
    let args = CliArgs::parse();
    let mut ctx = Ctx::default();

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
            CliArgs::command()
                .print_help()
                .map_err(CliError::from)
                .unwrap_or_else(|e| e.exit());

            exit(1);
        }
    }
}