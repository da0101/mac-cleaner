use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CandidateId, CleanupCandidate, FileIdentity, TargetId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanState {
    Prepared,
    Approved,
    Executing,
    Completed,
    Failed,
    Expired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    DryRun,
    Quarantine,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CleanupPlan {
    pub id: Uuid,
    pub catalog_version: u32,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub state: PlanState,
    pub execution_mode: ExecutionMode,
    pub items: Vec<CleanupCandidate>,
    pub total_bytes: u64,
    pub total_files: u64,
}

impl CleanupPlan {
    #[must_use]
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }
}

/// A capability constructed only after an exact prepared plan is confirmed.
///
/// This type is deliberately not serializable: the UI can see a plan, but it
/// cannot manufacture deletion authority.
#[derive(Debug)]
pub struct ApprovedPlan {
    plan: CleanupPlan,
    approval_id: Uuid,
    approved_at: DateTime<Utc>,
}

impl ApprovedPlan {
    #[must_use]
    pub fn new(plan: CleanupPlan, approved_at: DateTime<Utc>) -> Self {
        Self {
            plan,
            approval_id: Uuid::new_v4(),
            approved_at,
        }
    }

    #[must_use]
    pub fn plan(&self) -> &CleanupPlan {
        &self.plan
    }

    #[must_use]
    pub fn approval_id(&self) -> Uuid {
        self.approval_id
    }

    #[must_use]
    pub fn approved_at(&self) -> DateTime<Utc> {
        self.approved_at
    }

    #[must_use]
    pub fn into_plan(self) -> CleanupPlan {
        self.plan
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionItemReceipt {
    #[serde(default)]
    pub quarantine_entry_id: Option<Uuid>,
    pub candidate_id: CandidateId,
    #[serde(default)]
    pub target_id: Option<TargetId>,
    #[serde(default)]
    pub relative_entry: Option<PathBuf>,
    pub original_path: PathBuf,
    pub quarantine_path: Option<PathBuf>,
    pub bytes: u64,
    pub files: u64,
    pub completed_at: DateTime<Utc>,
    #[serde(default)]
    pub identity: Option<FileIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionReceipt {
    pub plan_id: Uuid,
    pub mode: ExecutionMode,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub items: Vec<ExecutionItemReceipt>,
}
