use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::FileIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCleanupKind {
    UvCachePrune,
    SimulatorTestRecordings,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCleanupState {
    Prepared,
    Approved,
    Executing,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCleanupFile {
    pub relative_name: String,
    pub identity: FileIdentity,
    pub allocated_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ToolCleanupOperation {
    Command {
        executable_path: PathBuf,
        arguments: Vec<String>,
    },
    SimulatorAttachments {
        device_udid: String,
        device_name: String,
        device_was_booted: bool,
        files: Vec<ToolCleanupFile>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCleanupPlan {
    pub id: Uuid,
    pub kind: ToolCleanupKind,
    pub state: ToolCleanupState,
    pub target_path: PathBuf,
    pub target_identity: FileIdentity,
    pub operation: ToolCleanupOperation,
    pub bytes_before: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ToolCleanupReceipt {
    pub plan_id: Uuid,
    pub kind: ToolCleanupKind,
    pub target_path: PathBuf,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub reclaimed_bytes: u64,
    pub completed_at: DateTime<Utc>,
    pub summary: String,
}
