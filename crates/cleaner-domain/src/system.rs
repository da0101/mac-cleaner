use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryPressureLevel {
    #[default]
    Normal,
    Warning,
    Critical,
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CpuPressureLevel {
    #[default]
    Normal,
    Warning,
    Critical,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CpuSnapshot {
    pub usage_percent: f32,
    pub logical_cores: u32,
    #[serde(default)]
    pub pressure: CpuPressureLevel,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub total_bytes: u64,
    #[serde(default)]
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub swap_used_bytes: u64,
    #[serde(default)]
    pub reclaimable_bytes: u64,
    #[serde(default)]
    pub compressed_bytes: u64,
    #[serde(default)]
    pub file_backed_bytes: u64,
    #[serde(default)]
    pub pressure: MemoryPressureLevel,
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
    pub cpu: CpuSnapshot,
    pub memory: MemorySnapshot,
    pub disk: DiskSnapshot,
    pub processes: Vec<ProcessSnapshot>,
}
