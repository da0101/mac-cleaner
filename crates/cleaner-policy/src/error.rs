use std::path::PathBuf;

use cleaner_domain::DomainError;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("home path is not a safe absolute user directory: {0}")]
    InvalidHome(PathBuf),
    #[error("catalog target has an invalid relative root: {0}")]
    InvalidCatalogRoot(String),
    #[error("candidate has an unsafe relative path: {0}")]
    InvalidCandidatePath(PathBuf),
    #[error("quarantine entry is no longer active: {0}")]
    QuarantineNotActive(Uuid),
    #[error("path is protected: {0}")]
    ProtectedPath(PathBuf),
    #[error("candidate is hard-linked: {0}")]
    HardLinked(PathBuf),
    #[error("candidate ID is unknown or no longer present: {0}")]
    UnknownCandidate(String),
    #[error("candidate ID appears more than once")]
    DuplicateCandidate,
    #[error("cleanup plan has {0} candidates; the maximum is 100")]
    TooManyCandidates(usize),
    #[error("candidate used catalog version {0}, which is no longer current")]
    StaleCatalog(u32),
    #[error("cleanup plan TTL cannot be represented")]
    InvalidPlanTtl,
    #[error("auto-clean settings rejected: {0}")]
    InvalidAutoSettings(String),
    #[error("cleanup plan impact is too large to represent")]
    ImpactOverflow,
}
