use std::path::PathBuf;

use cleaner_policy::PolicyError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error(transparent)]
    Policy(#[from] PolicyError),
    #[error("filesystem operation failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("unsafe cleanup candidate {path}: {reason}")]
    UnsafeCandidate { path: PathBuf, reason: String },
    #[error("home directory is not a safe real directory: {0}")]
    UnsafeHome(PathBuf),
    #[error("cleanup target is no longer in the catalog: {0}")]
    UnknownTarget(String),
    #[error("cleanup candidate changed after preview: {0}")]
    CandidateChanged(PathBuf),
    #[error("cleanup plan expired before execution")]
    ExpiredPlan,
    #[error("filesystem timestamp cannot be represented")]
    InvalidTimestamp,
    #[error("quarantine path is unsafe: {0}")]
    UnsafeQuarantine(PathBuf),
    #[error("quarantine entry already exists: {0}")]
    QuarantineCollision(PathBuf),
    #[error("quarantine entry changed after it was recorded: {0}")]
    QuarantineChanged(PathBuf),
    #[error("cannot restore because the original path is occupied: {0}")]
    RestoreDestinationOccupied(PathBuf),
    #[error(
        "failed to roll back {quarantine} to {original}; manual recovery is required: {source}"
    )]
    RollbackFailed {
        original: PathBuf,
        quarantine: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
