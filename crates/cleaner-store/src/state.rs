use chrono::{DateTime, Utc};
use cleaner_domain::PlanState;
use rusqlite::{Transaction, params};
use uuid::Uuid;

use crate::StoreError;

pub(crate) fn transition_rows(
    transaction: &Transaction<'_>,
    plan_id: Uuid,
    expected: PlanState,
    next: PlanState,
    approval: Option<(Uuid, DateTime<Utc>)>,
    error: Option<&str>,
) -> Result<(), StoreError> {
    let (approval_id, approved_at) = approval.map_or((None, None), |(id, at)| {
        (Some(id.to_string()), Some(at.to_rfc3339()))
    });
    let changed = transaction.execute(
        "UPDATE plans SET state = ?1, updated_at = ?2,
         approval_id = COALESCE(?3, approval_id),
         approved_at = COALESCE(?4, approved_at), error = ?5
         WHERE id = ?6 AND state = ?7",
        params![
            state_name(next),
            Utc::now().to_rfc3339(),
            approval_id,
            approved_at,
            error,
            plan_id.to_string(),
            state_name(expected),
        ],
    )?;
    if changed != 1 {
        return Err(StoreError::InvalidTransition {
            plan_id,
            expected,
            next,
        });
    }
    Ok(())
}

pub(crate) fn append_event(
    transaction: &Transaction<'_>,
    plan_id: Uuid,
    state: PlanState,
    detail: Option<&str>,
) -> Result<(), StoreError> {
    transaction.execute(
        "INSERT INTO plan_events (plan_id, state, occurred_at, detail) VALUES (?1, ?2, ?3, ?4)",
        params![
            plan_id.to_string(),
            state_name(state),
            Utc::now().to_rfc3339(),
            detail,
        ],
    )?;
    Ok(())
}

pub(crate) const fn state_name(state: PlanState) -> &'static str {
    match state {
        PlanState::Prepared => "prepared",
        PlanState::Approved => "approved",
        PlanState::Executing => "executing",
        PlanState::Completed => "completed",
        PlanState::Failed => "failed",
        PlanState::Expired => "expired",
    }
}

pub(crate) fn parse_state(value: &str) -> Result<PlanState, StoreError> {
    match value {
        "prepared" => Ok(PlanState::Prepared),
        "approved" => Ok(PlanState::Approved),
        "executing" => Ok(PlanState::Executing),
        "completed" => Ok(PlanState::Completed),
        "failed" => Ok(PlanState::Failed),
        "expired" => Ok(PlanState::Expired),
        _ => Err(StoreError::InvalidState(value.to_owned())),
    }
}
