use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use chrono::Utc;
use cleaner_domain::{
    STORAGE_CATALOG_VERSION, StorageAnalysis, StorageComparisonState, StorageDisposition,
    StorageHotspot, StoragePressure,
};
use uuid::Uuid;

use crate::FsError;
use crate::metadata::{metadata_no_follow, validate_home_state};
use crate::storage_catalog::{HotspotSpec, fixed_specs};
use crate::storage_project_catalog::{ProjectKind, project_specs, project_test_specs};

const MAX_ENTRIES_PER_HOTSPOT: u64 = 500_000;
const CRITICAL_BYTES: u64 = 10 * 1024 * 1024 * 1024;
const LOW_BYTES: u64 = 30 * 1024 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct StorageAnalyzer {
    home: PathBuf,
    max_entries: u64,
}

impl StorageAnalyzer {
    #[must_use]
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            max_entries: MAX_ENTRIES_PER_HOTSPOT,
        }
    }

    #[must_use]
    pub fn with_max_entries(mut self, max_entries: u64) -> Self {
        self.max_entries = max_entries;
        self
    }

    pub fn analyze(
        &self,
        total_bytes: u64,
        available_bytes: u64,
    ) -> Result<StorageAnalysis, FsError> {
        validate_home_state(&self.home)?;
        let home_device = metadata_no_follow(&self.home)?.dev();
        let mut hotspots = Vec::new();
        let mut warnings = Vec::new();

        for spec in fixed_specs() {
            self.add_hotspot(&mut hotspots, &mut warnings, home_device, spec);
        }
        if let Some(spec) = crate::storage_catalog::system_temp_spec() {
            self.add_hotspot(&mut hotspots, &mut warnings, home_device, spec);
        }
        self.inspect_projects(&mut hotspots, &mut warnings, home_device);
        hotspots.sort_by_key(|item| std::cmp::Reverse(item.bytes));
        let reviewable_bytes = hotspots
            .iter()
            .filter(|item| item.disposition == StorageDisposition::Reviewable)
            .fold(0_u64, |total, item| total.saturating_add(item.bytes));
        let pressure = if available_bytes < CRITICAL_BYTES {
            StoragePressure::Critical
        } else if available_bytes < LOW_BYTES {
            StoragePressure::Low
        } else {
            StoragePressure::Healthy
        };

        Ok(StorageAnalysis {
            snapshot_id: Uuid::new_v4(),
            catalog_version: STORAGE_CATALOG_VERSION,
            captured_at: Utc::now(),
            total_bytes,
            available_bytes,
            pressure,
            reviewable_bytes,
            hotspots,
            warnings,
            comparison: None,
        })
    }

    fn inspect_projects(
        &self,
        hotspots: &mut Vec<StorageHotspot>,
        warnings: &mut Vec<String>,
        device: u64,
    ) {
        let projects = self.home.join("Documents/GitHub");
        let Ok(entries) = fs::read_dir(&projects) else {
            return;
        };
        for entry in entries.flatten() {
            let root = entry.path();
            if !is_real_directory_on_device(&root, device) {
                continue;
            }
            self.inspect_project_root(hotspots, warnings, device, &root);
            if let Ok(children) = fs::read_dir(&root) {
                for child in children.flatten() {
                    let child = child.path();
                    if is_real_directory_on_device(&child, device) {
                        self.inspect_project_root(hotspots, warnings, device, &child);
                    }
                }
            }
        }
    }

    fn inspect_project_root(
        &self,
        hotspots: &mut Vec<StorageHotspot>,
        warnings: &mut Vec<String>,
        device: u64,
        root: &Path,
    ) {
        if root.join("pubspec.yaml").is_file() {
            for spec in project_specs(root, ProjectKind::Flutter) {
                self.add_hotspot(hotspots, warnings, device, spec);
            }
        }
        if root.join("package.json").is_file() {
            for spec in project_specs(root, ProjectKind::Node) {
                self.add_hotspot(hotspots, warnings, device, spec);
            }
        }
        if root.join("Cargo.toml").is_file() {
            for spec in project_specs(root, ProjectKind::Rust) {
                self.add_hotspot(hotspots, warnings, device, spec);
            }
        }
        if root.join("pyproject.toml").is_file() || root.join("requirements.txt").is_file() {
            let cache = root.join(".cache");
            if cache.exists() {
                self.add_hotspot(
                    hotspots,
                    warnings,
                    device,
                    HotspotSpec::project_cache(cache),
                );
            }
        }
        for spec in project_test_specs(root) {
            self.add_hotspot(hotspots, warnings, device, spec);
        }
    }

    fn add_hotspot(
        &self,
        hotspots: &mut Vec<StorageHotspot>,
        warnings: &mut Vec<String>,
        device: u64,
        spec: HotspotSpec,
    ) {
        let path = if spec.path.is_absolute() {
            spec.path
        } else {
            self.home.join(spec.path)
        };
        if !path.exists() {
            return;
        }
        match allocated_size(&path, device, self.max_entries) {
            Ok(bytes) if bytes > 0 => hotspots.push(StorageHotspot {
                id: stable_id(&path),
                display_name: spec.display_name,
                path,
                bytes,
                category: spec.category,
                disposition: spec.disposition,
                comparison_state: StorageComparisonState::NoBaseline,
                previous_bytes: None,
                delta_bytes: None,
                reason: spec.reason,
                recommendation: spec.recommendation,
                recovery_cost: spec.recovery_cost,
            }),
            Ok(_) => {}
            Err(reason) => warnings.push(format!("{}: {reason}", path.display())),
        }
    }
}

fn is_real_directory_on_device(path: &Path, device: u64) -> bool {
    metadata_no_follow(path).is_ok_and(|metadata| {
        metadata.is_dir() && !metadata.file_type().is_symlink() && metadata.dev() == device
    })
}

fn allocated_size(root: &Path, device: u64, max_entries: u64) -> Result<u64, String> {
    let mut stack = vec![root.to_path_buf()];
    let mut entries = 0_u64;
    let mut bytes = 0_u64;
    while let Some(path) = stack.pop() {
        if entries >= max_entries {
            return Err("hotspot exceeds the read-only entry limit".into());
        }
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.dev() != device {
            continue;
        }
        entries += 1;
        bytes = bytes.saturating_add(metadata.blocks().saturating_mul(512));
        if metadata.is_dir() {
            let children = fs::read_dir(&path).map_err(|error| error.to_string())?;
            for child in children.flatten() {
                stack.push(child.path());
            }
        }
    }
    Ok(bytes)
}

fn stable_id(path: &Path) -> String {
    format!("storage:{}", path.to_string_lossy())
}
