use std::collections::BTreeMap;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{Store, StoreError};

const MAX_SETTINGS_BYTES: u64 = 1_000_000;
const MAX_HISTORY_BYTES: u64 = 10_000_000;

struct ParsedSettings {
    values: BTreeMap<String, Value>,
    imported: Vec<String>,
    ignored: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LegacyHistoryEntry {
    pub timestamp: Option<String>,
    pub items_cleaned: Option<u64>,
    pub total_cleaned: Option<String>,
    pub total_cleaned_bytes: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MigrationSummary {
    pub source_root: PathBuf,
    pub already_imported: bool,
    pub imported_settings: Vec<String>,
    pub ignored_settings: Vec<String>,
    pub imported_history_entries: u64,
}

impl Store {
    pub fn import_legacy(&self, source_root: &Path) -> Result<MigrationSummary, StoreError> {
        let source_root = validate_source_root(source_root)?;
        let fingerprint = fingerprint(&source_root);
        let existing: Option<String> = self
            .connection
            .lock()
            .query_row(
                "SELECT summary_json FROM imports WHERE source_fingerprint = ?1",
                [&fingerprint],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(summary) = existing {
            let mut summary: MigrationSummary = serde_json::from_str(&summary)?;
            summary.already_imported = true;
            return Ok(summary);
        }

        let settings = read_settings(&source_root)?;
        let history = read_history(&source_root)?;
        let imported_history_entries =
            u64::try_from(history.len()).map_err(|_| StoreError::CountOverflow)?;
        let summary = MigrationSummary {
            source_root: source_root.clone(),
            already_imported: false,
            imported_settings: settings.imported.clone(),
            ignored_settings: settings.ignored,
            imported_history_entries,
        };

        let mut connection = self.connection.lock();
        let transaction = connection.transaction()?;
        for (key, value) in settings.values {
            transaction.execute(
                "INSERT INTO settings (key, value_json, updated_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(key) DO NOTHING",
                params![key, serde_json::to_string(&value)?, Utc::now().to_rfc3339()],
            )?;
        }
        for entry in history {
            transaction.execute(
                "INSERT INTO legacy_history (source_fingerprint, occurred_at, summary_json)
                 VALUES (?1, ?2, ?3)",
                params![fingerprint, entry.timestamp, serde_json::to_string(&entry)?,],
            )?;
        }
        transaction.execute(
            "INSERT INTO imports (source_fingerprint, imported_at, summary_json)
             VALUES (?1, ?2, ?3)",
            params![
                fingerprint,
                Utc::now().to_rfc3339(),
                serde_json::to_string(&summary)?,
            ],
        )?;
        transaction.commit()?;
        Ok(summary)
    }

    pub fn legacy_history(&self, limit: u32) -> Result<Vec<LegacyHistoryEntry>, StoreError> {
        let connection = self.connection.lock();
        let mut statement = connection
            .prepare("SELECT summary_json FROM legacy_history ORDER BY id DESC LIMIT ?1")?;
        let rows = statement.query_map([limit.min(100)], |row| row.get::<_, String>(0))?;
        let mut history = Vec::new();
        for row in rows {
            history.push(serde_json::from_str(&row?)?);
        }
        Ok(history)
    }
}

fn validate_source_root(source_root: &Path) -> Result<PathBuf, StoreError> {
    if !source_root.is_absolute() || source_root == Path::new("/") {
        return Err(StoreError::InvalidMigrationSource(
            source_root.to_path_buf(),
        ));
    }
    let metadata = fs::symlink_metadata(source_root).map_err(|source| StoreError::MigrationIo {
        path: source_root.to_path_buf(),
        source,
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(StoreError::InvalidMigrationSource(
            source_root.to_path_buf(),
        ));
    }
    source_root
        .canonicalize()
        .map_err(|source| StoreError::MigrationIo {
            path: source_root.to_path_buf(),
            source,
        })
}

fn read_settings(source_root: &Path) -> Result<ParsedSettings, StoreError> {
    let Some(value) = read_json(source_root, "settings.json", MAX_SETTINGS_BYTES)? else {
        return Ok(ParsedSettings {
            values: BTreeMap::new(),
            imported: Vec::new(),
            ignored: Vec::new(),
        });
    };
    let object = value
        .as_object()
        .ok_or_else(|| StoreError::InvalidMigrationJson("settings.json".to_owned()))?;
    let mut settings = BTreeMap::new();
    let mut ignored = Vec::new();
    for (key, value) in object {
        let destination = match key.as_str() {
            "theme" if matches!(value.as_str(), Some("light" | "dark" | "system")) => Some("theme"),
            "dashboard_status_interval_seconds"
                if value
                    .as_u64()
                    .is_some_and(|seconds| (5..=300).contains(&seconds)) =>
            {
                Some("monitor_interval_seconds")
            }
            "show_cleanup_history" if value.is_boolean() => Some("show_cleanup_history"),
            "show_garbage_breakdown" if value.is_boolean() => Some("show_garbage_breakdown"),
            "show_ram_breakdown" if value.is_boolean() => Some("show_ram_breakdown"),
            _ => None,
        };
        if let Some(destination) = destination {
            settings.insert(destination.to_owned(), value.clone());
        } else if key != "$schema" {
            ignored.push(key.clone());
        }
    }
    let imported = settings.keys().cloned().collect();
    ignored.sort();
    Ok(ParsedSettings {
        values: settings,
        imported,
        ignored,
    })
}

fn read_history(source_root: &Path) -> Result<Vec<LegacyHistoryEntry>, StoreError> {
    let Some(value) = read_json(source_root, "cleanup_history.json", MAX_HISTORY_BYTES)? else {
        return Ok(Vec::new());
    };
    let entries = value
        .as_array()
        .ok_or_else(|| StoreError::InvalidMigrationJson("cleanup_history.json".to_owned()))?;
    Ok(entries
        .iter()
        .take(100)
        .filter_map(|value| {
            let object = value.as_object()?;
            Some(LegacyHistoryEntry {
                timestamp: object
                    .get("timestamp")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                items_cleaned: object.get("items_cleaned").and_then(Value::as_u64),
                total_cleaned: object
                    .get("total_cleaned")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                total_cleaned_bytes: object.get("total_cleaned_bytes").and_then(Value::as_u64),
            })
        })
        .collect())
}

fn read_json(source_root: &Path, name: &str, max_bytes: u64) -> Result<Option<Value>, StoreError> {
    let path = source_root.join(name);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(StoreError::MigrationIo { path, source }),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > max_bytes {
        return Err(StoreError::InvalidMigrationFile(path));
    }
    let contents = fs::read_to_string(&path).map_err(|source| StoreError::MigrationIo {
        path: path.clone(),
        source,
    })?;
    Ok(Some(serde_json::from_str(&contents)?))
}

fn fingerprint(path: &Path) -> String {
    let mut digest = Sha256::new();
    digest.update(path.as_os_str().as_bytes());
    format!("{:x}", digest.finalize())
}
