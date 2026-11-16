use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const STORAGE_CATALOG_VERSION: u32 = 2;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageCategory {
    ContainerData,
    AppleDevelopment,
    AndroidDevelopment,
    PackageCache,
    ProjectDependencies,
    ProjectBuild,
    TestArtifacts,
    ApplicationSupport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageComparisonState {
    NoBaseline,
    Comparable,
    NewlyTracked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageComparisonKind {
    Previous,
    Baseline,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageHotspot {
    pub id: String,
    pub display_name: String,
    pub path: PathBuf,
    pub bytes: u64,
    pub category: StorageCategory,
    pub disposition: StorageDisposition,
    pub comparison_state: StorageComparisonState,
    pub previous_bytes: Option<u64>,
    pub delta_bytes: Option<i64>,
    pub reason: String,
    pub recommendation: String,
    pub recovery_cost: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageComparison {
    pub snapshot_id: Uuid,
    pub captured_at: DateTime<Utc>,
    pub kind: StorageComparisonKind,
    pub volume_used_delta_bytes: i64,
    pub measured_hotspot_delta_bytes: i64,
    pub accounting_gap_bytes: i64,
    pub comparable_hotspots: u32,
    pub newly_tracked_hotspots: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageAnalysis {
    pub snapshot_id: Uuid,
    pub catalog_version: u32,
    pub captured_at: DateTime<Utc>,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub pressure: StoragePressure,
    pub reviewable_bytes: u64,
    pub hotspots: Vec<StorageHotspot>,
    pub warnings: Vec<String>,
    pub comparison: Option<StorageComparison>,
}

impl StorageAnalysis {
    #[must_use]
    pub fn used_bytes(&self) -> u64 {
        self.total_bytes.saturating_sub(self.available_bytes)
    }

    pub fn compare_with(&mut self, previous: Option<(&Self, StorageComparisonKind)>) {
        self.comparison = None;
        for hotspot in &mut self.hotspots {
            hotspot.comparison_state = StorageComparisonState::NoBaseline;
            hotspot.previous_bytes = None;
            hotspot.delta_bytes = None;
        }
        let Some((previous, kind)) = previous else {
            return;
        };
        let previous_by_id: HashMap<&str, u64> = previous
            .hotspots
            .iter()
            .map(|hotspot| (hotspot.id.as_str(), hotspot.bytes))
            .collect();
        let mut measured_delta = 0_i64;
        let mut comparable = 0_u32;
        let mut newly_tracked = 0_u32;
        for hotspot in &mut self.hotspots {
            if let Some(previous_bytes) = previous_by_id.get(hotspot.id.as_str()).copied() {
                let delta = signed_delta(hotspot.bytes, previous_bytes);
                hotspot.comparison_state = StorageComparisonState::Comparable;
                hotspot.previous_bytes = Some(previous_bytes);
                hotspot.delta_bytes = Some(delta);
                measured_delta = measured_delta.saturating_add(delta);
                comparable = comparable.saturating_add(1);
            } else {
                hotspot.comparison_state = StorageComparisonState::NewlyTracked;
                newly_tracked = newly_tracked.saturating_add(1);
            }
        }
        let volume_delta = signed_delta(self.used_bytes(), previous.used_bytes());
        self.comparison = Some(StorageComparison {
            snapshot_id: previous.snapshot_id,
            captured_at: previous.captured_at,
            kind,
            volume_used_delta_bytes: volume_delta,
            measured_hotspot_delta_bytes: measured_delta,
            accounting_gap_bytes: volume_delta.saturating_sub(measured_delta),
            comparable_hotspots: comparable,
            newly_tracked_hotspots: newly_tracked,
        });
    }
}

fn signed_delta(current: u64, previous: u64) -> i64 {
    if current >= previous {
        i64::try_from(current - previous).unwrap_or(i64::MAX)
    } else {
        -i64::try_from(previous - current).unwrap_or(i64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analysis(used: u64, hotspots: &[(&str, u64)]) -> StorageAnalysis {
        StorageAnalysis {
            snapshot_id: Uuid::new_v4(),
            catalog_version: STORAGE_CATALOG_VERSION,
            captured_at: Utc::now(),
            total_bytes: 1_000,
            available_bytes: 1_000 - used,
            pressure: StoragePressure::Healthy,
            reviewable_bytes: 0,
            hotspots: hotspots
                .iter()
                .map(|(id, bytes)| StorageHotspot {
                    id: (*id).to_owned(),
                    display_name: (*id).to_owned(),
                    path: PathBuf::from(format!("/{id}")),
                    bytes: *bytes,
                    category: StorageCategory::PackageCache,
                    disposition: StorageDisposition::Reviewable,
                    comparison_state: StorageComparisonState::NoBaseline,
                    previous_bytes: None,
                    delta_bytes: None,
                    reason: String::new(),
                    recommendation: String::new(),
                    recovery_cost: String::new(),
                })
                .collect(),
            warnings: Vec::new(),
            comparison: None,
        }
    }

    #[test]
    fn comparison_reports_signed_growth_and_accounting_gap() {
        let previous = analysis(400, &[("same", 100), ("gone", 40)]);
        let mut current = analysis(550, &[("same", 170), ("new", 30)]);

        current.compare_with(Some((&previous, StorageComparisonKind::Previous)));

        let Some(comparison) = current.comparison.as_ref() else {
            panic!("comparison should be present");
        };
        assert_eq!(comparison.volume_used_delta_bytes, 150);
        assert_eq!(comparison.measured_hotspot_delta_bytes, 70);
        assert_eq!(comparison.accounting_gap_bytes, 80);
        assert_eq!(current.hotspots[0].delta_bytes, Some(70));
        assert_eq!(
            current.hotspots[1].comparison_state,
            StorageComparisonState::NewlyTracked
        );
    }

    #[test]
    fn first_scan_has_no_invented_deltas() {
        let mut current = analysis(500, &[("only", 100)]);
        current.compare_with(None);
        assert!(current.comparison.is_none());
        assert_eq!(
            current.hotspots[0].comparison_state,
            StorageComparisonState::NoBaseline
        );
    }
}
