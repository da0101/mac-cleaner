use chrono::{DateTime, Utc};
use cleaner_domain::{
    CleanupPlan, ExecutionMode, ExecutionReceipt, PlanState, QuarantineEntry, QuarantineState,
};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::state::{append_event, parse_state, state_name, transition_rows};
use crate::{Store, StoreError};

impl Store {
    pub fn save_plan(&self, plan: &CleanupPlan) -> Result<(), StoreError> {
        let plan_json = serde_json::to_string(plan)?;
        let now = Utc::now().to_rfc3339();
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO plans
             (id, state, created_at, expires_at, plan_json, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                plan.id.to_string(),
                state_name(plan.state),
                plan.created_at.to_rfc3339(),
                plan.expires_at.to_rfc3339(),
                plan_json,
                now,
            ],
        )?;
        append_event(&transaction, plan.id, PlanState::Prepared, None)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn load_plan(&self, plan_id: Uuid) -> Result<Option<CleanupPlan>, StoreError> {
        let connection = self.connection.lock();
        let value: Option<String> = connection
            .query_row(
                "SELECT plan_json FROM plans WHERE id = ?1",
                [plan_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        value
            .map(|json| serde_json::from_str(&json).map_err(StoreError::from))
            .transpose()
    }

    pub fn mark_approved(
        &self,
        plan_id: Uuid,
        approval_id: Uuid,
        approved_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        self.transition(
            plan_id,
            PlanState::Prepared,
            PlanState::Approved,
            Some((approval_id, approved_at)),
            None,
        )
    }

    pub fn mark_executing(&self, plan_id: Uuid) -> Result<(), StoreError> {
        self.transition(
            plan_id,
            PlanState::Approved,
            PlanState::Executing,
            None,
            None,
        )
    }

    pub fn mark_failed(&self, plan_id: Uuid, error: &str) -> Result<(), StoreError> {
        let current = self.current_state(plan_id)?;
        self.transition(plan_id, current, PlanState::Failed, None, Some(error))
    }

    pub fn save_receipt(&self, receipt: &ExecutionReceipt) -> Result<(), StoreError> {
        let receipt_json = serde_json::to_string(receipt)?;
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transition_rows(
            &transaction,
            receipt.plan_id,
            PlanState::Executing,
            PlanState::Completed,
            None,
            None,
        )?;
        transaction.execute(
            "INSERT INTO receipts (plan_id, completed_at, receipt_json) VALUES (?1, ?2, ?3)",
            params![
                receipt.plan_id.to_string(),
                receipt.completed_at.to_rfc3339(),
                receipt_json,
            ],
        )?;
        if receipt.mode == ExecutionMode::Quarantine {
            for item in &receipt.items {
                let entry = QuarantineEntry {
                    id: item.quarantine_entry_id.ok_or_else(|| {
                        StoreError::IncompleteQuarantineReceipt(item.candidate_id.0.clone())
                    })?,
                    plan_id: receipt.plan_id,
                    candidate_id: item.candidate_id.clone(),
                    target_id: item.target_id.clone().ok_or_else(|| {
                        StoreError::IncompleteQuarantineReceipt(item.candidate_id.0.clone())
                    })?,
                    relative_entry: item.relative_entry.clone().ok_or_else(|| {
                        StoreError::IncompleteQuarantineReceipt(item.candidate_id.0.clone())
                    })?,
                    original_path: item.original_path.clone(),
                    quarantine_path: item.quarantine_path.clone().ok_or_else(|| {
                        StoreError::IncompleteQuarantineReceipt(item.candidate_id.0.clone())
                    })?,
                    bytes: item.bytes,
                    files: item.files,
                    identity: item.identity.clone().ok_or_else(|| {
                        StoreError::IncompleteQuarantineReceipt(item.candidate_id.0.clone())
                    })?,
                    quarantined_at: item.completed_at,
                    state: QuarantineState::Active,
                    updated_at: item.completed_at,
                };
                transaction.execute(
                    "INSERT INTO quarantine_entries
                     (id, plan_id, state, entry_json, updated_at)
                     VALUES (?1, ?2, 'active', ?3, ?4)",
                    params![
                        entry.id.to_string(),
                        entry.plan_id.to_string(),
                        serde_json::to_string(&entry)?,
                        entry.updated_at.to_rfc3339(),
                    ],
                )?;
            }
        }
        append_event(&transaction, receipt.plan_id, PlanState::Completed, None)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn recent_receipts(&self, limit: u32) -> Result<Vec<ExecutionReceipt>, StoreError> {
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare("SELECT receipt_json FROM receipts ORDER BY completed_at DESC LIMIT ?1")?;
        let rows = statement.query_map([limit], |row| row.get::<_, String>(0))?;
        let mut receipts = Vec::new();
        for row in rows {
            receipts.push(serde_json::from_str(&row?)?);
        }
        Ok(receipts)
    }

    fn current_state(&self, plan_id: Uuid) -> Result<PlanState, StoreError> {
        let connection = self.connection.lock();
        let state: String = connection
            .query_row(
                "SELECT state FROM plans WHERE id = ?1",
                [plan_id.to_string()],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StoreError::PlanNotFound(plan_id))?;
        parse_state(&state)
    }

    fn transition(
        &self,
        plan_id: Uuid,
        expected: PlanState,
        next: PlanState,
        approval: Option<(Uuid, DateTime<Utc>)>,
        error: Option<&str>,
    ) -> Result<(), StoreError> {
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transition_rows(&transaction, plan_id, expected, next, approval, error)?;
        append_event(&transaction, plan_id, next, error)?;
        transaction.commit()?;
        Ok(())
    }
}
