use chrono::{DateTime, Utc};
use cleaner_domain::{PlanState, QuarantineActionPlan, QuarantineEntry, QuarantineState};
use rusqlite::{OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::state::{parse_state, state_name};
use crate::{Store, StoreError};

impl Store {
    pub fn active_quarantine(&self, limit: u32) -> Result<Vec<QuarantineEntry>, StoreError> {
        let connection = self.connection.lock();
        let mut statement = connection.prepare(
            "SELECT entry_json FROM quarantine_entries
             WHERE state = 'active' ORDER BY updated_at DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| row.get::<_, String>(0))?;
        let mut entries = Vec::new();
        for row in rows {
            entries.push(serde_json::from_str(&row?)?);
        }
        Ok(entries)
    }

    pub fn active_quarantine_entry(&self, entry_id: Uuid) -> Result<QuarantineEntry, StoreError> {
        let json: Option<String> = self
            .connection
            .lock()
            .query_row(
                "SELECT entry_json FROM quarantine_entries WHERE id = ?1 AND state = 'active'",
                [entry_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(StoreError::from))
            .transpose()?
            .ok_or(StoreError::QuarantineEntryNotFound(entry_id))
    }

    pub fn save_quarantine_action(&self, plan: &QuarantineActionPlan) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO quarantine_actions
             (id, entry_id, state, action_json, updated_at)
             VALUES (?1, ?2, 'prepared', ?3, ?4)",
            params![
                plan.id.to_string(),
                plan.entry.id.to_string(),
                serde_json::to_string(plan)?,
                now,
            ],
        )?;
        append_event(&transaction, plan.id, PlanState::Prepared, None)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn load_quarantine_action(
        &self,
        action_id: Uuid,
    ) -> Result<QuarantineActionPlan, StoreError> {
        let connection = self.connection.lock();
        let row: Option<(String, String)> = connection
            .query_row(
                "SELECT action_json, state FROM quarantine_actions WHERE id = ?1",
                [action_id.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (json, state) = row.ok_or(StoreError::QuarantineActionNotFound(action_id))?;
        let mut plan: QuarantineActionPlan = serde_json::from_str(&json)?;
        plan.state = parse_state(&state)?;
        Ok(plan)
    }

    pub fn mark_quarantine_action_approved(
        &self,
        action_id: Uuid,
        approval_id: Uuid,
        approved_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        self.transition_action(
            action_id,
            PlanState::Prepared,
            PlanState::Approved,
            Some((approval_id, approved_at)),
            None,
        )
    }

    pub fn mark_quarantine_action_executing(&self, action_id: Uuid) -> Result<(), StoreError> {
        self.transition_action(
            action_id,
            PlanState::Approved,
            PlanState::Executing,
            None,
            None,
        )
    }

    pub fn complete_quarantine_action(
        &self,
        action_id: Uuid,
        entry: &QuarantineEntry,
    ) -> Result<(), StoreError> {
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transition_rows(
            &transaction,
            action_id,
            PlanState::Executing,
            PlanState::Completed,
            None,
            None,
        )?;
        let changed = transaction.execute(
            "UPDATE quarantine_entries SET state = ?1, entry_json = ?2, updated_at = ?3
             WHERE id = ?4 AND state = 'active'",
            params![
                quarantine_state_name(entry.state),
                serde_json::to_string(entry)?,
                entry.updated_at.to_rfc3339(),
                entry.id.to_string(),
            ],
        )?;
        if changed != 1 {
            return Err(StoreError::QuarantineEntryNotFound(entry.id));
        }
        append_event(&transaction, action_id, PlanState::Completed, None)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn fail_quarantine_action(&self, action_id: Uuid, error: &str) -> Result<(), StoreError> {
        let plan = self.load_quarantine_action(action_id)?;
        self.transition_action(action_id, plan.state, PlanState::Failed, None, Some(error))
    }

    fn transition_action(
        &self,
        action_id: Uuid,
        expected: PlanState,
        next: PlanState,
        approval: Option<(Uuid, DateTime<Utc>)>,
        error: Option<&str>,
    ) -> Result<(), StoreError> {
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transition_rows(&transaction, action_id, expected, next, approval, error)?;
        append_event(&transaction, action_id, next, error)?;
        transaction.commit()?;
        Ok(())
    }
}

fn transition_rows(
    transaction: &Transaction<'_>,
    action_id: Uuid,
    expected: PlanState,
    next: PlanState,
    approval: Option<(Uuid, DateTime<Utc>)>,
    error: Option<&str>,
) -> Result<(), StoreError> {
    let (approval_id, approved_at) = approval.map_or((None, None), |(id, at)| {
        (Some(id.to_string()), Some(at.to_rfc3339()))
    });
    let changed = transaction.execute(
        "UPDATE quarantine_actions SET state = ?1, updated_at = ?2,
         approval_id = COALESCE(?3, approval_id),
         approved_at = COALESCE(?4, approved_at), error = ?5
         WHERE id = ?6 AND state = ?7",
        params![
            state_name(next),
            Utc::now().to_rfc3339(),
            approval_id,
            approved_at,
            error,
            action_id.to_string(),
            state_name(expected),
        ],
    )?;
    if changed != 1 {
        return Err(StoreError::InvalidTransition {
            plan_id: action_id,
            expected,
            next,
        });
    }
    Ok(())
}

fn append_event(
    transaction: &Transaction<'_>,
    action_id: Uuid,
    state: PlanState,
    detail: Option<&str>,
) -> Result<(), StoreError> {
    transaction.execute(
        "INSERT INTO quarantine_action_events (action_id, state, occurred_at, detail)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            action_id.to_string(),
            state_name(state),
            Utc::now().to_rfc3339(),
            detail,
        ],
    )?;
    Ok(())
}

const fn quarantine_state_name(state: QuarantineState) -> &'static str {
    match state {
        QuarantineState::Active => "active",
        QuarantineState::Restored => "restored",
        QuarantineState::Purged => "purged",
    }
}
