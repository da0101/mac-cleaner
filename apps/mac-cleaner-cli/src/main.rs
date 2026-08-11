mod app;
mod cli;
mod commands;
mod confirmation;
mod output;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("mac_cleaner=info")
        .with_target(false)
        .compact()
        .init();

    app::run(Cli::parse()).await
}
