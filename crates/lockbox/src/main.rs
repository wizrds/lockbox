mod cli;
mod manifest;
mod constants;
mod config;

use lockbox_core::telemetry::bootstrap;

#[cfg(all(feature = "mimalloc", not(feature = "stdalloc")))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> anyhow::Result<()> {
    bootstrap("lockbox", |telemetry| async move {
        cli::CliArgs::run(telemetry).await;
        Ok(())
    })
}
