use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    #[serde(default)]
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub swap_used_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiskSnapshot {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub start_time_unix: u64,
    pub name: String,
    pub executable: Option<String>,
    pub memory_bytes: u64,
    pub cpu_percent: f32,
    pub protected: bool,
    pub protection_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub captured_at: DateTime<Utc>,
    pub memory: MemorySnapshot,
    pub disk: DiskSnapshot,
    pub processes: Vec<ProcessSnapshot>,
}
