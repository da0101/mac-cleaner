use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Version of the compiled cleanup catalog used to discover a candidate.
pub const CATALOG_VERSION: u32 = 1;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TargetId(pub String);

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CandidateId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SafetyTier {
    SafeAuto,
    ReviewRequired,
    Protected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileKind {
    File,
    Directory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryMode {
    Rebuild,
    Redownload,
    QuarantineRestore,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileIdentity {
    pub device: u64,
    pub inode: u64,
    pub size: u64,
    pub modified_unix_nanos: i64,
    pub link_count: u64,
    pub kind: FileKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupCandidate {
    pub id: CandidateId,
    pub target_id: TargetId,
    pub display_name: String,
    pub lexical_path: PathBuf,
    pub resolved_path: PathBuf,
    pub relative_entry: PathBuf,
    pub category: String,
    pub reason: String,
    pub owner: String,
    pub recovery: String,
    pub recovery_mode: RecoveryMode,
    pub safety_tier: SafetyTier,
    pub bytes: u64,
    pub files: u64,
    pub last_modified: DateTime<Utc>,
    pub identity: FileIdentity,
    pub catalog_version: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScanSkip {
    pub target_id: TargetId,
    pub path: PathBuf,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScanReport {
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub candidates: Vec<CleanupCandidate>,
    pub skipped: Vec<ScanSkip>,
}
