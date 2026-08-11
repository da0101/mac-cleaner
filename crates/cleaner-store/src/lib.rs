//! `SQLite` settings, cleanup plan journal, and migration support.

use std::path::{Path, PathBuf};

use parking_lot::Mutex;
use rusqlite::Connection;

mod cleanup_plans;
mod error;
mod migration;
mod quarantine;
mod recovery;
mod schema;
mod settings;
mod state;
mod tool_cleanup;

pub use error::StoreError;
pub use migration::{LegacyHistoryEntry, MigrationSummary};
use schema::SCHEMA;

pub struct Store {
    connection: Mutex<Connection>,
    path: Option<PathBuf>,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| StoreError::CreateDirectory {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let connection = Connection::open(path)?;
        let store = Self {
            connection: Mutex::new(connection),
            path: Some(path.to_path_buf()),
        };
        store.initialize()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let store = Self {
            connection: Mutex::new(Connection::open_in_memory()?),
            path: None,
        };
        store.initialize()?;
        Ok(store)
    }

    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn initialize(&self) -> Result<(), StoreError> {
        let connection = self.connection.lock();
        connection.busy_timeout(std::time::Duration::from_secs(5))?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;")?;
        connection.execute_batch(SCHEMA)?;
        Ok(())
    }
}
