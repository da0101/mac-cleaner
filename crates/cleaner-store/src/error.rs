use std::path::PathBuf;

use cleaner_domain::PlanState;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("cannot create storage directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cleanup plan was not found: {0}")]
    PlanNotFound(Uuid),
    #[error("quarantine entry was not found or is no longer active: {0}")]
    QuarantineEntryNotFound(Uuid),
    #[error("quarantine action was not found: {0}")]
    QuarantineActionNotFound(Uuid),
    #[error("tool cleanup plan was not found: {0}")]
    ToolCleanupPlanNotFound(Uuid),
    #[error("invalid tool cleanup transition for {plan_id}: expected {expected}, requested {next}")]
    InvalidToolCleanupTransition {
        plan_id: Uuid,
        expected: String,
        next: String,
    },
    #[error("cleanup receipt is missing quarantine identity data for candidate {0}")]
    IncompleteQuarantineReceipt(String),
    #[error("invalid plan transition for {plan_id}: expected {expected:?}, requested {next:?}")]
    InvalidTransition {
        plan_id: Uuid,
        expected: PlanState,
        next: PlanState,
    },
    #[error("database contains an unknown plan state: {0}")]
    InvalidState(String),
    #[error("database change count cannot be represented")]
    CountOverflow,
    #[error("legacy migration source is not a safe absolute directory: {0}")]
    InvalidMigrationSource(PathBuf),
    #[error("legacy migration file is a link, special file, or too large: {0}")]
    InvalidMigrationFile(PathBuf),
    #[error("legacy migration JSON has the wrong shape: {0}")]
    InvalidMigrationJson(String),
    #[error("cannot read legacy migration source {path}: {source}")]
    MigrationIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
