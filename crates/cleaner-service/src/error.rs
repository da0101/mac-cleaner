use std::path::PathBuf;

use thiserror::Error;
use uuid::Uuid;

/// Failures reported by the application orchestration layer.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Policy(#[from] cleaner_policy::PolicyError),
    #[error(transparent)]
    Filesystem(#[from] cleaner_fs::FsError),
    #[error(transparent)]
    Store(#[from] cleaner_store::StoreError),
    #[error(transparent)]
    System(#[from] cleaner_system_macos::SystemError),
    #[error("background task failed: {0}")]
    TaskJoin(#[from] tokio::task::JoinError),
    #[error("a scan is already running")]
    ScanInProgress,
    #[error("a storage analysis is already running")]
    AnalysisInProgress,
    #[error("a cleanup operation is already running")]
    MutationInProgress,
    #[error("run a fresh scan before preparing a cleanup plan")]
    ScanRequired,
    #[error("cleanup plan was not found: {0}")]
    PlanNotFound(Uuid),
    #[error("cleanup plan was already approved: {0}")]
    AlreadyApproved(Uuid),
    #[error("cleanup plan has no live single-use approval: {0}")]
    ApprovalMissing(Uuid),
    #[error("application data cannot be stored in a cleanup cache: {0}")]
    UnsafeDataDirectory(PathBuf),
    #[error("tool cleanup plan expired: {0}")]
    ToolCleanupExpired(Uuid),
    #[error("tool cleanup target is unsafe: {0}")]
    UnsafeToolTarget(String),
    #[error("cannot execute owning tool {path}: {source}")]
    ToolExecution {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("owning tool cleanup failed: {0}")]
    ToolFailed(String),
    #[error("active developer test job detected ({0}); stop the test and try again")]
    ActiveDeveloperJob(String),
    #[error("cannot parse Simulator metadata: {0}")]
    SimulatorMetadata(#[source] serde_json::Error),
    #[error("cannot prepare application data directory {path}: {source}")]
    DataDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("cannot acquire mutation lock {path}: {source}")]
    LockFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
