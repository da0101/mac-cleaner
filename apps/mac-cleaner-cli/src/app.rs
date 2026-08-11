use anyhow::{Context, Result};
use cleaner_service::AppService;
use directories::{BaseDirs, ProjectDirs};

use crate::cli::{Cli, Command};
use crate::commands::{cleanup, migration, quarantine, reporting};

pub(crate) async fn run(cli: Cli) -> Result<()> {
    let service = open_service()?;

    match cli.command {
        Command::Status { json } => reporting::run_status(&service, json).await?,
        Command::Scan { json } => reporting::run_scan(&service, json).await?,
        Command::Clean { candidate, dry_run } => {
            cleanup::run(&service, candidate, dry_run).await?;
        }
        Command::History { limit, json } => reporting::run_history(&service, limit, json)?,
        Command::Quarantine { command } => quarantine::run(&service, command).await?,
        Command::Migrate { from, json } => migration::run(&service, &from, json)?,
    }

    Ok(())
}

fn open_service() -> Result<AppService> {
    let base = BaseDirs::new().context("macOS home directory is unavailable")?;
    let project = ProjectDirs::from("com", "da0101", "Mac Cleaner")
        .context("macOS application data directory is unavailable")?;
    AppService::open(
        base.home_dir().to_path_buf(),
        project.data_dir().to_path_buf(),
    )
    .context("cannot initialize Mac Cleaner")
}
