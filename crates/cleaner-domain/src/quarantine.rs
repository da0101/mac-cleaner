use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{CandidateId, FileIdentity, PlanState, TargetId};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineState {
    Active,
    Restored,
    Purged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuarantineEntry {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub candidate_id: CandidateId,
    pub target_id: TargetId,
    pub relative_entry: PathBuf,
    pub original_path: PathBuf,
    pub quarantine_path: PathBuf,
    pub bytes: u64,
    pub files: u64,
    pub identity: FileIdentity,
    pub quarantined_at: DateTime<Utc>,
    pub state: QuarantineState,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineAction {
    Restore,
    Purge,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct QuarantineActionPlan {
    pub id: Uuid,
    pub action: QuarantineAction,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub state: PlanState,
    pub entry: QuarantineEntry,
}

impl QuarantineActionPlan {
    #[must_use]
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now >= self.expires_at
    }
}

/// Backend-only, single-use authority for one quarantine mutation.
#[derive(Debug)]
pub struct ApprovedQuarantineAction {
    plan: QuarantineActionPlan,
    approval_id: Uuid,
    approved_at: DateTime<Utc>,
}

impl ApprovedQuarantineAction {
    #[must_use]
    pub fn new(plan: QuarantineActionPlan, approved_at: DateTime<Utc>) -> Self {
        Self {
            plan,
            approval_id: Uuid::new_v4(),
            approved_at,
        }
    }

    #[must_use]
    pub fn plan(&self) -> &QuarantineActionPlan {
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
    pub fn into_plan(self) -> QuarantineActionPlan {
        self.plan
    }
}
