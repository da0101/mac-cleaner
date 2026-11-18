//! Per-app view of CPU and memory use, for the read-only Energy screen.

use serde::{Deserialize, Serialize};

/// One app (or one tool) with all of its helper processes added together.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppEnergy {
    pub name: String,
    /// The `.app` bundle, when the group is a real application.
    pub app_path: Option<String>,
    pub process_count: u32,
    pub memory_bytes: u64,
    /// 100 equals one fully used logical core.
    pub cpu_percent: f64,
    pub protected: bool,
    pub protection_reason: Option<String>,
    /// True only when Mac Cleaner may ask this app to quit normally.
    pub can_quit: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnergyReport {
    pub logical_cores: u32,
    pub total_cpu_percent: f64,
    pub apps: Vec<AppEnergy>,
}
