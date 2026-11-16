use cleaner_domain::StorageAnalysis;
use rusqlite::{OptionalExtension, params};
use uuid::Uuid;

use crate::{Store, StoreError};

const BASELINE_SETTING: &str = "storage_baseline_snapshot_id";
const MAX_SNAPSHOTS: u32 = 30;

impl Store {
    pub fn save_storage_snapshot(&self, snapshot: &StorageAnalysis) -> Result<(), StoreError> {
        let json = serde_json::to_string(snapshot)?;
        self.connection.lock().execute(
            "INSERT INTO storage_snapshots (id, captured_at, snapshot_json)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET captured_at = excluded.captured_at,
             snapshot_json = excluded.snapshot_json",
            params![
                snapshot.snapshot_id.to_string(),
                snapshot.captured_at.to_rfc3339(),
                json
            ],
        )?;
        self.prune_storage_snapshots()
    }

    pub fn latest_storage_snapshot(&self) -> Result<Option<StorageAnalysis>, StoreError> {
        self.load_storage_query(
            "SELECT snapshot_json FROM storage_snapshots ORDER BY captured_at DESC LIMIT 1",
            [],
        )
    }

    pub fn storage_snapshot(&self, id: Uuid) -> Result<Option<StorageAnalysis>, StoreError> {
        self.load_storage_query(
            "SELECT snapshot_json FROM storage_snapshots WHERE id = ?1",
            [id.to_string()],
        )
    }

    pub fn storage_snapshot_count(&self) -> Result<u32, StoreError> {
        let count = self.connection.lock().query_row(
            "SELECT COUNT(*) FROM storage_snapshots",
            [],
            |row| row.get::<_, u32>(0),
        )?;
        Ok(count)
    }

    pub fn set_storage_baseline(&self, id: Uuid) -> Result<(), StoreError> {
        if self.storage_snapshot(id)?.is_none() {
            return Err(StoreError::StorageSnapshotNotFound(id));
        }
        self.set_setting(BASELINE_SETTING, &id)?;
        self.prune_storage_snapshots()
    }

    pub fn storage_baseline(&self) -> Result<Option<StorageAnalysis>, StoreError> {
        let Some(id) = self.get_setting::<Uuid>(BASELINE_SETTING)? else {
            return Ok(None);
        };
        self.storage_snapshot(id)
    }

    fn load_storage_query<P>(
        &self,
        sql: &str,
        params: P,
    ) -> Result<Option<StorageAnalysis>, StoreError>
    where
        P: rusqlite::Params,
    {
        let json: Option<String> = self
            .connection
            .lock()
            .query_row(sql, params, |row| row.get(0))
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(StoreError::from))
            .transpose()
    }

    fn prune_storage_snapshots(&self) -> Result<(), StoreError> {
        let baseline = self.get_setting::<Uuid>(BASELINE_SETTING)?;
        let keep_recent = if baseline.is_some() {
            MAX_SNAPSHOTS - 1
        } else {
            MAX_SNAPSHOTS
        };
        self.connection.lock().execute(
            "DELETE FROM storage_snapshots
             WHERE id NOT IN (
                 SELECT id FROM storage_snapshots ORDER BY captured_at DESC LIMIT ?1
             ) AND (?2 IS NULL OR id != ?2)",
            params![keep_recent, baseline.map(|id| id.to_string())],
        )?;
        Ok(())
    }
}
