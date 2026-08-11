use std::collections::{HashMap, HashSet};
use std::time::Duration;

use chrono::{DateTime, Utc};
use cleaner_domain::{
    ApprovedPlan, CATALOG_VERSION, CandidateId, CleanupCandidate, CleanupPlan, DomainError,
    ExecutionMode, FileKind, PlanState, SafetyTier,
};
use uuid::Uuid;

use crate::{PolicyError, is_safe_relative};

const MAX_PLAN_ITEMS: usize = 100;

pub fn prepare_plan(
    candidates: &[CleanupCandidate],
    selected_ids: &[CandidateId],
    now: DateTime<Utc>,
    ttl: Duration,
    execution_mode: ExecutionMode,
) -> Result<CleanupPlan, PolicyError> {
    if selected_ids.is_empty() {
        return Err(PolicyError::Domain(DomainError::EmptyPlan));
    }
    if selected_ids.len() > MAX_PLAN_ITEMS {
        return Err(PolicyError::TooManyCandidates(selected_ids.len()));
    }

    let unique: HashSet<&CandidateId> = selected_ids.iter().collect();
    if unique.len() != selected_ids.len() {
        return Err(PolicyError::DuplicateCandidate);
    }

    let by_id: HashMap<&CandidateId, &CleanupCandidate> = candidates
        .iter()
        .map(|candidate| (&candidate.id, candidate))
        .collect();
    let mut items = Vec::with_capacity(selected_ids.len());
    for selected_id in selected_ids {
        let candidate = by_id
            .get(selected_id)
            .ok_or_else(|| PolicyError::UnknownCandidate(selected_id.0.clone()))?;
        validate_candidate(candidate)?;
        items.push((*candidate).clone());
    }

    let total_bytes = checked_total(items.iter().map(|item| item.bytes))?;
    let total_files = checked_total(items.iter().map(|item| item.files))?;
    let ttl = chrono::Duration::from_std(ttl).map_err(|_| PolicyError::InvalidPlanTtl)?;

    Ok(CleanupPlan {
        id: Uuid::new_v4(),
        catalog_version: CATALOG_VERSION,
        created_at: now,
        expires_at: now + ttl,
        state: PlanState::Prepared,
        execution_mode,
        items,
        total_bytes,
        total_files,
    })
}

pub fn approve_plan(
    mut plan: CleanupPlan,
    now: DateTime<Utc>,
) -> Result<ApprovedPlan, PolicyError> {
    if plan.state != PlanState::Prepared {
        return Err(PolicyError::Domain(DomainError::PlanNotPrepared));
    }
    if plan.is_expired(now) {
        return Err(PolicyError::Domain(DomainError::ExpiredPlan));
    }
    if plan.items.is_empty() {
        return Err(PolicyError::Domain(DomainError::EmptyPlan));
    }
    for candidate in &plan.items {
        validate_candidate(candidate)?;
    }
    plan.state = PlanState::Approved;
    Ok(ApprovedPlan::new(plan, now))
}

fn checked_total(mut values: impl Iterator<Item = u64>) -> Result<u64, PolicyError> {
    values.try_fold(0_u64, |total, value| {
        total.checked_add(value).ok_or(PolicyError::ImpactOverflow)
    })
}

fn validate_candidate(candidate: &CleanupCandidate) -> Result<(), PolicyError> {
    if candidate.catalog_version != CATALOG_VERSION {
        return Err(PolicyError::StaleCatalog(candidate.catalog_version));
    }
    if candidate.safety_tier == SafetyTier::Protected {
        return Err(PolicyError::ProtectedPath(candidate.lexical_path.clone()));
    }
    if candidate.identity.kind == FileKind::File && candidate.identity.link_count > 1 {
        return Err(PolicyError::HardLinked(candidate.lexical_path.clone()));
    }
    if !is_safe_relative(&candidate.relative_entry) {
        return Err(PolicyError::InvalidCandidatePath(
            candidate.relative_entry.clone(),
        ));
    }
    Ok(())
}
