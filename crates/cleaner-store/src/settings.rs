use chrono::Utc;
use rusqlite::{OptionalExtension, params};

use crate::{Store, StoreError};

impl Store {
    pub fn set_setting<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), StoreError> {
        let json = serde_json::to_string(value)?;
        self.connection.lock().execute(
            "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
             updated_at = excluded.updated_at",
            params![key, json, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn get_setting<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StoreError> {
        let json: Option<String> = self
            .connection
            .lock()
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(StoreError::from))
            .transpose()
    }
}
