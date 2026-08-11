use chrono::Utc;

use crate::{Store, StoreError};

impl Store {
    pub fn fail_incomplete_on_startup(&self) -> Result<u64, StoreError> {
        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        let now = Utc::now().to_rfc3339();
        let updated = transaction.execute(
            "UPDATE plans SET state = 'failed', updated_at = ?1,
             error = 'application restarted before the operation completed'
             WHERE state IN ('approved', 'executing')",
            [&now],
        )?;
        let quarantine_updated = transaction.execute(
            "UPDATE quarantine_actions SET state = 'failed', updated_at = ?1,
             error = 'application restarted before the quarantine action completed'
             WHERE state IN ('approved', 'executing')",
            [&now],
        )?;
        let tool_updated = transaction.execute(
            "UPDATE tool_cleanup_plans SET state = 'failed', updated_at = ?1,
             error = 'application restarted before the tool cleanup completed'
             WHERE state IN ('approved', 'executing')",
            [&now],
        )?;
        transaction.execute(
            "INSERT INTO plan_events (plan_id, state, occurred_at, detail)
             SELECT id, 'failed', ?1, 'startup recovery: deletion was not resumed'
             FROM plans WHERE updated_at = ?1 AND state = 'failed'",
            [&now],
        )?;
        transaction.execute(
            "INSERT INTO quarantine_action_events (action_id, state, occurred_at, detail)
             SELECT id, 'failed', ?1, 'startup recovery: quarantine action was not resumed'
             FROM quarantine_actions WHERE updated_at = ?1 AND state = 'failed'",
            [&now],
        )?;
        transaction.execute(
            "INSERT INTO tool_cleanup_events (plan_id, state, occurred_at, detail)
             SELECT id, 'failed', ?1, 'startup recovery: tool cleanup was not resumed'
             FROM tool_cleanup_plans WHERE updated_at = ?1 AND state = 'failed'",
            [&now],
        )?;
        transaction.commit()?;
        u64::try_from(
            updated
                .saturating_add(quarantine_updated)
                .saturating_add(tool_updated),
        )
        .map_err(|_| StoreError::CountOverflow)
    }
}
