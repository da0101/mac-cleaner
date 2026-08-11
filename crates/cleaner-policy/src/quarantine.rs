use std::time::Duration;

use chrono::{DateTime, Utc};
use cleaner_domain::{
    ApprovedQuarantineAction, DomainError, PlanState, QuarantineAction, QuarantineActionPlan,
    QuarantineEntry, QuarantineState,
};
use uuid::Uuid;

use crate::{PolicyError, is_safe_relative};

pub fn prepare_quarantine_action(
    entry: QuarantineEntry,
    action: QuarantineAction,
    now: DateTime<Utc>,
    ttl: Duration,
) -> Result<QuarantineActionPlan, PolicyError> {
    if entry.state != QuarantineState::Active {
        return Err(PolicyError::QuarantineNotActive(entry.id));
    }
    if !is_safe_relative(&entry.relative_entry) {
        return Err(PolicyError::InvalidCandidatePath(entry.relative_entry));
    }
    let ttl = chrono::Duration::from_std(ttl).map_err(|_| PolicyError::InvalidPlanTtl)?;
    Ok(QuarantineActionPlan {
        id: Uuid::new_v4(),
        action,
        created_at: now,
        expires_at: now + ttl,
        state: PlanState::Prepared,
        entry,
    })
}

pub fn approve_quarantine_action(
    mut plan: QuarantineActionPlan,
    now: DateTime<Utc>,
) -> Result<ApprovedQuarantineAction, PolicyError> {
    if plan.state != PlanState::Prepared {
        return Err(PolicyError::Domain(DomainError::PlanNotPrepared));
    }
    if plan.is_expired(now) {
        return Err(PolicyError::Domain(DomainError::ExpiredPlan));
    }
    if plan.entry.state != QuarantineState::Active {
        return Err(PolicyError::QuarantineNotActive(plan.entry.id));
    }
    plan.state = PlanState::Approved;
    Ok(ApprovedQuarantineAction::new(plan, now))
}
