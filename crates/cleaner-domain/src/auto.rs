//! Auto-clean settings, decisions, and activity records.
//!
//! Auto-clean runs only inside limits the owner has opted into: a list of apps
//! that may be asked to quit, and a list of cleanup categories that may be
//! removed on a schedule. Everything else stays protected.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const GIB: u64 = 1_073_741_824;

/// When to relieve memory pressure by asking opted-in apps to quit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoRamSettings {
    pub enabled: bool,
    /// Act when conservative available memory stays below this many bytes.
    pub min_available_bytes: u64,
    /// How long memory must stay low before anything is quit.
    pub sustained_secs: u64,
    /// Minimum gap between two quit requests.
    pub cooldown_secs: u64,
    /// Display names of apps (`Google Chrome`) that may be asked to quit.
    pub quit_apps: Vec<String>,
}

impl Default for AutoRamSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            min_available_bytes: 2 * GIB,
            sustained_secs: 120,
            cooldown_secs: 180,
            quit_apps: Vec::new(),
        }
    }
}

/// Scheduled removal of opted-in cleanup categories.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoStorageSettings {
    pub enabled: bool,
    /// Normal gap between runs.
    pub interval_hours: u64,
    /// Also run (at most hourly) while free disk space is below this.
    pub low_free_bytes: u64,
    /// Items changed more recently than this are never removed.
    pub min_idle_hours: u64,
    /// Catalog target IDs that may be removed (`npm-content-cache`, …).
    pub target_ids: Vec<String>,
}

impl Default for AutoStorageSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_hours: 6,
            low_free_bytes: 40 * GIB,
            min_idle_hours: 24,
            target_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoCleanSettings {
    #[serde(default)]
    pub ram: AutoRamSettings,
    #[serde(default)]
    pub storage: AutoStorageSettings,
}

/// One app, found by its bundle path, that the policy chose to quit.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoQuitTarget {
    pub app_name: String,
    pub app_path: String,
    pub bytes: u64,
    pub pids: Vec<u32>,
}

/// What the RAM policy wants done right now.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutoRamDecision {
    Disabled,
    Healthy,
    Waiting { secs_remaining: u64 },
    Cooldown { secs_remaining: u64 },
    NoTargets { reason: String },
    Quit { target: AutoQuitTarget },
}

/// Persistent bookkeeping for the RAM policy.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoRamState {
    pub low_since: Option<DateTime<Utc>>,
    pub last_quit_at: Option<DateTime<Utc>>,
    /// Recent quit requests, newest last, used to stop retrying an app that refuses.
    #[serde(default)]
    pub recent_quits: Vec<(String, DateTime<Utc>)>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoLogKind {
    RamQuit,
    StorageClean,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoLogEntry {
    pub at: DateTime<Utc>,
    pub kind: AutoLogKind,
    pub summary: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoStorageState {
    pub last_run: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoStorageReport {
    pub items: u64,
    pub bytes_freed: u64,
    pub skipped: u64,
}

/// Result of one engine tick.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AutoTickReport {
    pub ram: AutoRamDecision,
    pub storage: Option<AutoStorageReport>,
}
