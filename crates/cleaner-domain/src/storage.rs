use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StoragePressure {
    Critical,
    Low,
    Healthy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageDisposition {
    Reviewable,
    ToolManaged,
    Protected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageHotspot {
    pub id: String,
    pub display_name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub disposition: StorageDisposition,
    pub reason: String,
    pub recommendation: String,
    pub recovery_cost: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageAnalysis {
    pub captured_at: DateTime<Utc>,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub pressure: StoragePressure,
    pub reviewable_bytes: u64,
    pub hotspots: Vec<StorageHotspot>,
    pub warnings: Vec<String>,
}
