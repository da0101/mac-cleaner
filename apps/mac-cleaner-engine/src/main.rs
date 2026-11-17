use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use cleaner_domain::{AutoCleanSettings, CandidateId, ExecutionMode, QuarantineAction};
use cleaner_service::AppService;
use directories::{BaseDirs, ProjectDirs};
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Parser)]
#[command(
    name = "mac-cleaner-engine",
    version,
    about = "Mac Cleaner native app engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Status,
    StorageAnalyze,
    StorageSetBaseline {
        snapshot: Uuid,
    },
    Scan,
    PrepareClean {
        #[arg(long, required = true)]
        candidate: Vec<String>,
        #[arg(long)]
        dry_run: bool,
    },
    ExecuteClean {
        plan: Uuid,
    },
    History {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    QuarantineList {
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    PrepareQuarantine {
        entry: Uuid,
        action: EngineQuarantineAction,
    },
    ExecuteQuarantine {
        action: Uuid,
    },
    PrepareUvPrune,
    PrepareSimulatorRecordings,
    ExecuteToolCleanup {
        plan: Uuid,
    },
    ToolHistory {
        #[arg(long, default_value_t = 20)]
        limit: u32,
    },
    /// Current auto-clean settings (everything off until the owner opts in).
    AutoSettings,
    /// Replace the auto-clean settings; Rust validates and clamps them.
    AutoSettingsSet {
        json: String,
    },
    /// One scheduler step: RAM-relief decision plus any due storage clean.
    AutoTick,
    /// Record that an opted-in app was asked to quit.
    AutoReportQuit {
        app: String,
        bytes: u64,
    },
    AutoLog {
        #[arg(long, default_value_t = 30)]
        limit: u32,
    },
    /// Names of apps that can never be put on the quit list.
    AutoProtectedApps,
}

const MAX_AUTO_SETTINGS_BYTES: usize = 16 * 1024;
const MAX_APP_NAME_BYTES: usize = 80;

#[derive(Clone, Copy, Debug, ValueEnum)]
enum EngineQuarantineAction {
    Restore,
    Purge,
}

impl From<EngineQuarantineAction> for QuarantineAction {
    fn from(value: EngineQuarantineAction) -> Self {
        match value {
            EngineQuarantineAction::Restore => Self::Restore,
            EngineQuarantineAction::Purge => Self::Purge,
        }
    }
}

#[derive(Serialize)]
struct Success<T> {
    protocol_version: u32,
    ok: bool,
    data: T,
}

#[derive(Serialize)]
struct Failure<'a> {
    protocol_version: u32,
    ok: bool,
    error: &'a str,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(data) => match serde_json::to_string(&Success {
            protocol_version: PROTOCOL_VERSION,
            ok: true,
            data,
        }) {
            Ok(output) => {
                println!("{output}");
                ExitCode::SUCCESS
            }
            Err(error) => fail(&error.to_string()),
        },
        Err(error) => fail(&format!("{error:#}")),
    }
}

fn fail(message: &str) -> ExitCode {
    let output = serde_json::to_string(&Failure {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        error: message,
    })
    .unwrap_or_else(|_| {
        "{\"protocol_version\":1,\"ok\":false,\"error\":\"engine failure\"}".into()
    });
    eprintln!("{output}");
    ExitCode::FAILURE
}

async fn run(cli: Cli) -> Result<Value> {
    let service = open_service()?;
    let data = match cli.command {
        Command::Status => serde_json::to_value(service.system_snapshot().await?)?,
        Command::StorageAnalyze => serde_json::to_value(service.analyze_storage().await?)?,
        Command::StorageSetBaseline { snapshot } => {
            service.set_storage_baseline(snapshot)?;
            json!({ "snapshot_id": snapshot })
        }
        Command::Scan => serde_json::to_value(service.scan().await?)?,
        Command::PrepareClean { candidate, dry_run } => {
            service.scan().await?;
            let ids = candidate.into_iter().map(CandidateId).collect::<Vec<_>>();
            let mode = if dry_run {
                ExecutionMode::DryRun
            } else {
                ExecutionMode::Quarantine
            };
            serde_json::to_value(service.prepare_plan(&ids, mode)?)?
        }
        Command::ExecuteClean { plan } => {
            service.approve_plan(plan)?;
            serde_json::to_value(service.execute_approved(plan).await?)?
        }
        Command::History { limit } => serde_json::to_value(service.recent_history(limit)?)?,
        Command::QuarantineList { limit } => {
            serde_json::to_value(service.active_quarantine(limit)?)?
        }
        Command::PrepareQuarantine { entry, action } => {
            serde_json::to_value(service.prepare_quarantine_action(entry, action.into())?)?
        }
        Command::ExecuteQuarantine { action } => {
            service.approve_quarantine_action(action)?;
            serde_json::to_value(service.execute_quarantine_action(action).await?)?
        }
        Command::PrepareUvPrune => serde_json::to_value(service.prepare_uv_cache_prune().await?)?,
        Command::PrepareSimulatorRecordings => {
            serde_json::to_value(service.prepare_simulator_recordings_cleanup().await?)?
        }
        Command::ExecuteToolCleanup { plan } => {
            service.approve_tool_cleanup(plan)?;
            serde_json::to_value(service.execute_tool_cleanup(plan).await?)?
        }
        Command::ToolHistory { limit } => {
            serde_json::to_value(service.recent_tool_cleanup_receipts(limit)?)?
        }
        Command::AutoSettings => serde_json::to_value(service.auto_settings()?)?,
        Command::AutoSettingsSet { json } => {
            anyhow::ensure!(
                json.len() <= MAX_AUTO_SETTINGS_BYTES,
                "auto-clean settings are too large"
            );
            let settings: AutoCleanSettings =
                serde_json::from_str(&json).context("auto-clean settings are not valid")?;
            serde_json::to_value(service.set_auto_settings(settings)?)?
        }
        Command::AutoTick => serde_json::to_value(service.auto_tick().await?)?,
        Command::AutoReportQuit { app, bytes } => {
            anyhow::ensure!(
                !app.is_empty() && app.len() <= MAX_APP_NAME_BYTES,
                "app name has an invalid length"
            );
            service.auto_report_quit(&app, bytes)?;
            json!({ "recorded": true })
        }
        Command::AutoLog { limit } => serde_json::to_value(service.auto_log(limit)?)?,
        Command::AutoProtectedApps => serde_json::to_value(service.auto_protected_apps())?,
    };
    Ok(data)
}

fn open_service() -> Result<AppService> {
    let base = BaseDirs::new().context("macOS home directory is unavailable")?;
    let project = ProjectDirs::from("com", "da0101", "Mac Cleaner")
        .context("macOS application data directory is unavailable")?;
    AppService::open(
        base.home_dir().to_path_buf(),
        project.data_dir().to_path_buf(),
    )
    .context("cannot initialize Mac Cleaner engine")
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};

    #[test]
    fn prepare_cleanup_requires_opaque_candidate_ids() {
        assert!(Cli::try_parse_from(["engine", "prepare-clean"]).is_err());
        let parsed = Cli::try_parse_from([
            "engine",
            "prepare-clean",
            "--candidate",
            "catalog:item:identity",
        ]);
        assert!(matches!(
            parsed.map(|cli| cli.command),
            Ok(Command::PrepareClean { .. })
        ));
    }

    #[test]
    fn protocol_has_no_arbitrary_path_argument() {
        assert!(Cli::try_parse_from(["engine", "execute-clean", "--path", "/tmp/item"]).is_err());
    }
}
