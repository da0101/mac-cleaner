use chrono::{DateTime, Utc};
use cleaner_domain::{ToolCleanupKind, ToolCleanupPlan, ToolCleanupReceipt, ToolCleanupState};
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::{Store, StoreError};

impl Store {
    pub fn save_tool_cleanup_plan(&self, plan: &ToolCleanupPlan) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        transaction.execute(
            "INSERT INTO tool_cleanup_plans (id, state, plan_json, updated_at)
             VALUES (?1, 'prepared', ?2, ?3)",
            params![plan.id.to_string(), serde_json::to_string(plan)?, now],
        )?;
        transaction.execute(
            "INSERT INTO tool_cleanup_events (plan_id, state, occurred_at)
             VALUES (?1, 'prepared', ?2)",
            params![plan.id.to_string(), Utc::now().to_rfc3339()],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn load_tool_cleanup_plan(&self, plan_id: Uuid) -> Result<ToolCleanupPlan, StoreError> {
        let connection = self.connection.lock();
        let json: String = connection
            .query_row(
                "SELECT plan_json FROM tool_cleanup_plans WHERE id = ?1",
                [plan_id.to_string()],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StoreError::ToolCleanupPlanNotFound(plan_id))?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn mark_tool_cleanup_approved(
        &self,
        plan_id: Uuid,
        approval_id: Uuid,
        approved_at: DateTime<Utc>,
    ) -> Result<(), StoreError> {
        self.transition_tool_cleanup(
            plan_id,
            ToolCleanupState::Prepared,
            ToolCleanupState::Approved,
            Some((approval_id, approved_at)),
            None,
        )
    }

    pub fn mark_tool_cleanup_executing(&self, plan_id: Uuid) -> Result<(), StoreError> {
        self.transition_tool_cleanup(
            plan_id,
            ToolCleanupState::Approved,
            ToolCleanupState::Executing,
            None,
            None,
        )
    }

    pub fn append_tool_cleanup_detail(
        &self,
        plan_id: Uuid,
        detail: &str,
    ) -> Result<(), StoreError> {
        let inserted = self.connection.lock().execute(
            "INSERT INTO tool_cleanup_events (plan_id, state, occurred_at, detail)
             SELECT id, 'executing', ?1, ?2 FROM tool_cleanup_plans
             WHERE id = ?3 AND state = 'executing'",
            params![Utc::now().to_rfc3339(), detail, plan_id.to_string()],
        )?;
        if inserted != 1 {
            return Err(StoreError::InvalidToolCleanupTransition {
                plan_id,
                expected: "executing".to_owned(),
                next: "executing detail".to_owned(),
            });
        }
        Ok(())
    }

    pub fn fail_tool_cleanup(&self, plan_id: Uuid, error: &str) -> Result<(), StoreError> {
        let current = self.tool_cleanup_state(plan_id)?;
        self.transition_tool_cleanup(
            plan_id,
            current,
            ToolCleanupState::Failed,
            None,
            Some(error),
        )
    }

    pub fn save_tool_cleanup_receipt(
        &self,
        receipt: &ToolCleanupReceipt,
    ) -> Result<(), StoreError> {
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let updated = transaction.execute(
            "UPDATE tool_cleanup_plans SET state = 'completed', updated_at = ?1
             WHERE id = ?2 AND state = 'executing'",
            params![Utc::now().to_rfc3339(), receipt.plan_id.to_string()],
        )?;
        if updated != 1 {
            return Err(StoreError::InvalidToolCleanupTransition {
                plan_id: receipt.plan_id,
                expected: "executing".to_owned(),
                next: "completed".to_owned(),
            });
        }
        transaction.execute(
            "INSERT INTO tool_cleanup_receipts (plan_id, completed_at, receipt_json)
             VALUES (?1, ?2, ?3)",
            params![
                receipt.plan_id.to_string(),
                receipt.completed_at.to_rfc3339(),
                serde_json::to_string(receipt)?,
            ],
        )?;
        transaction.execute(
            "INSERT INTO tool_cleanup_events (plan_id, state, occurred_at, detail)
             VALUES (?1, 'completed', ?2, ?3)",
            params![
                receipt.plan_id.to_string(),
                receipt.completed_at.to_rfc3339(),
                receipt.summary,
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn latest_tool_cleanup_receipt(
        &self,
        kind: ToolCleanupKind,
    ) -> Result<Option<ToolCleanupReceipt>, StoreError> {
        let kind_name = serde_json::to_string(&kind)?;
        let connection = self.connection.lock();
        let receipt_json = connection
            .query_row(
                "SELECT r.receipt_json
                 FROM tool_cleanup_receipts r
                 JOIN tool_cleanup_plans p ON p.id = r.plan_id
                 WHERE json_extract(p.plan_json, '$.kind') = json_extract(?1, '$')
                   AND json_extract(r.receipt_json, '$.reclaimed_bytes') > 0
                 ORDER BY r.completed_at DESC LIMIT 1",
                [kind_name],
                |row| row.get::<_, String>(0),
            )
            .optional()?;
        receipt_json
            .map(|json| serde_json::from_str(&json).map_err(StoreError::from))
            .transpose()
    }

    pub fn recent_tool_cleanup_receipts(
        &self,
        limit: u32,
    ) -> Result<Vec<ToolCleanupReceipt>, StoreError> {
        let connection = self.connection.lock();
        let mut statement = connection.prepare(
            "SELECT receipt_json FROM tool_cleanup_receipts
             ORDER BY completed_at DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([limit], |row| row.get::<_, String>(0))?;
        rows.map(|row| {
            let json = row?;
            serde_json::from_str(&json).map_err(StoreError::from)
        })
        .collect()
    }

    fn tool_cleanup_state(&self, plan_id: Uuid) -> Result<ToolCleanupState, StoreError> {
        let connection = self.connection.lock();
        let state: String = connection
            .query_row(
                "SELECT state FROM tool_cleanup_plans WHERE id = ?1",
                [plan_id.to_string()],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(StoreError::ToolCleanupPlanNotFound(plan_id))?;
        parse_tool_state(&state)
    }

    fn transition_tool_cleanup(
        &self,
        plan_id: Uuid,
        expected: ToolCleanupState,
        next: ToolCleanupState,
        approval: Option<(Uuid, DateTime<Utc>)>,
        error: Option<&str>,
    ) -> Result<(), StoreError> {
        let expected_name = tool_state_name(expected);
        let next_name = tool_state_name(next);
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let (approval_id, approved_at) = approval.map_or((None, None), |(id, at)| {
            (Some(id.to_string()), Some(at.to_rfc3339()))
        });
        let updated = transaction.execute(
            "UPDATE tool_cleanup_plans
             SET state = ?1, approval_id = COALESCE(?2, approval_id),
                 approved_at = COALESCE(?3, approved_at), updated_at = ?4, error = ?5
             WHERE id = ?6 AND state = ?7",
            params![
                next_name,
                approval_id,
                approved_at,
                Utc::now().to_rfc3339(),
                error,
                plan_id.to_string(),
                expected_name,
            ],
        )?;
        if updated != 1 {
            return Err(StoreError::InvalidToolCleanupTransition {
                plan_id,
                expected: expected_name.to_owned(),
                next: next_name.to_owned(),
            });
        }
        transaction.execute(
            "INSERT INTO tool_cleanup_events (plan_id, state, occurred_at, detail)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                plan_id.to_string(),
                next_name,
                Utc::now().to_rfc3339(),
                error
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }
}

fn parse_tool_state(value: &str) -> Result<ToolCleanupState, StoreError> {
    match value {
        "prepared" => Ok(ToolCleanupState::Prepared),
        "approved" => Ok(ToolCleanupState::Approved),
        "executing" => Ok(ToolCleanupState::Executing),
        "completed" => Ok(ToolCleanupState::Completed),
        "failed" => Ok(ToolCleanupState::Failed),
        _ => Err(StoreError::InvalidState(value.to_owned())),
    }
}

const fn tool_state_name(state: ToolCleanupState) -> &'static str {
    match state {
        ToolCleanupState::Prepared => "prepared",
        ToolCleanupState::Approved => "approved",
        ToolCleanupState::Executing => "executing",
        ToolCleanupState::Completed => "completed",
        ToolCleanupState::Failed => "failed",
    }
}
