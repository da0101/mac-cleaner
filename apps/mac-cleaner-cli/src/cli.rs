use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "mac-cleaner",
    version,
    about = "Explainable, preview-first macOS cleanup"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Show read-only memory and disk health.
    Status {
        #[arg(long)]
        json: bool,
    },
    /// Scan exact versioned catalog roots without changing anything.
    Scan {
        #[arg(long)]
        json: bool,
    },
    /// Prepare, display, confirm, and execute one exact cleanup plan.
    Clean {
        /// Candidate ID from a fresh `scan`; repeat for multiple candidates.
        #[arg(long, required = true)]
        candidate: Vec<String>,
        /// Revalidate the plan without moving any files.
        #[arg(long)]
        dry_run: bool,
    },
    /// Show completed cleanup receipts, newest first.
    History {
        #[arg(long, default_value_t = 20)]
        limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Inspect, restore, or permanently purge recoverable quarantine entries.
    Quarantine {
        #[command(subcommand)]
        command: QuarantineCommand,
    },
    /// Copy safe settings and sanitized history from a legacy checkout.
    Migrate {
        /// Absolute legacy repository directory containing settings/history JSON.
        #[arg(long, value_name = "DIRECTORY")]
        from: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub(crate) enum QuarantineCommand {
    /// List entries still held in private quarantine.
    List {
        #[arg(long, default_value_t = 100)]
        limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Restore one exact quarantine entry to its original catalog path.
    Restore {
        #[arg(long)]
        entry: uuid::Uuid,
    },
    /// Permanently delete one exact quarantine entry and reclaim its disk space.
    Purge {
        #[arg(long)]
        entry: uuid::Uuid,
    },
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::Cli;

    #[test]
    fn clean_requires_candidate_ids() {
        assert!(Cli::try_parse_from(["mac-cleaner", "clean"]).is_err());
    }
}
