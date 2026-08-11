//! Read-only macOS disk, memory, and process metrics.

use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use cleaner_domain::{
    DiskSnapshot, MemoryPressureLevel, MemorySnapshot, ProcessSnapshot, SystemSnapshot,
};
use parking_lot::Mutex;
use sysinfo::{Disks, ProcessesToUpdate, System};
use thiserror::Error;

pub struct SystemMonitor {
    system: Mutex<System>,
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self {
            system: Mutex::new(System::new_all()),
        }
    }
}

impl SystemMonitor {
    pub fn capture(&self, path: &Path) -> Result<SystemSnapshot, SystemError> {
        let mut system = self.system.lock();
        system.refresh_memory();
        system.refresh_processes(ProcessesToUpdate::All, true);
        let processes = top_processes(&system);
        let memory = conservative_memory(&system);

        let disks = Disks::new_with_refreshed_list();
        let disk = disks
            .iter()
            .filter(|disk| path.starts_with(disk.mount_point()))
            .max_by_key(|disk| disk.mount_point().components().count())
            .ok_or_else(|| SystemError::DiskNotFound(path.to_path_buf()))?;

        Ok(SystemSnapshot {
            captured_at: Utc::now(),
            memory,
            disk: DiskSnapshot {
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
            },
            processes,
        })
    }
}

fn conservative_memory(system: &System) -> MemorySnapshot {
    let total_bytes = system.total_memory();
    let fallback_available = total_bytes.saturating_sub(system.used_memory());
    let vm = Command::new("/usr/bin/vm_stat")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|output| parse_vm_stat(&output));
    let free_bytes = vm.as_ref().map_or(system.free_memory(), |stats| stats.free);
    let available_bytes = vm
        .as_ref()
        .map_or(fallback_available, VmMemoryStats::available_bytes)
        .min(total_bytes);
    let swap_used_bytes = system.used_swap();

    MemorySnapshot {
        total_bytes,
        free_bytes,
        available_bytes,
        used_bytes: total_bytes.saturating_sub(available_bytes),
        swap_used_bytes,
        reclaimable_bytes: available_bytes.saturating_sub(free_bytes),
        compressed_bytes: vm.as_ref().map_or(0, |stats| stats.compressed),
        file_backed_bytes: vm.as_ref().map_or(0, |stats| stats.file_backed),
        pressure: classify_memory_pressure(available_bytes, total_bytes, swap_used_bytes),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VmMemoryStats {
    free: u64,
    inactive: u64,
    speculative: u64,
    purgeable: u64,
    compressed: u64,
    file_backed: u64,
}

impl VmMemoryStats {
    fn available_bytes(&self) -> u64 {
        self.free
            .saturating_add(self.inactive)
            .saturating_add(self.speculative)
            .saturating_add(self.purgeable)
    }
}

fn parse_vm_stat(output: &str) -> Option<VmMemoryStats> {
    let mut lines = output.lines();
    let page_size = lines
        .next()?
        .split_whitespace()
        .find_map(|word| word.parse::<u64>().ok())?;
    let mut free_pages = 0_u64;
    let mut inactive_pages = 0_u64;
    let mut speculative_pages = 0_u64;
    let mut purgeable_pages = 0_u64;
    let mut compressed_pages = 0_u64;
    let mut file_backed_pages = 0_u64;

    for line in lines {
        let Some((name, raw_value)) = line.split_once(':') else {
            continue;
        };
        let pages = raw_value.trim().trim_end_matches('.').parse::<u64>().ok()?;
        match name.trim() {
            "Pages free" => free_pages = pages,
            "Pages inactive" => inactive_pages = pages,
            "Pages speculative" => speculative_pages = pages,
            "Pages purgeable" => purgeable_pages = pages,
            "Pages occupied by compressor" => compressed_pages = pages,
            "File-backed pages" => file_backed_pages = pages,
            _ => {}
        }
    }

    Some(VmMemoryStats {
        free: free_pages.saturating_mul(page_size),
        inactive: inactive_pages.saturating_mul(page_size),
        speculative: speculative_pages.saturating_mul(page_size),
        purgeable: purgeable_pages.saturating_mul(page_size),
        compressed: compressed_pages.saturating_mul(page_size),
        file_backed: file_backed_pages.saturating_mul(page_size),
    })
}

fn classify_memory_pressure(
    available_bytes: u64,
    total_bytes: u64,
    swap_used_bytes: u64,
) -> MemoryPressureLevel {
    if total_bytes == 0 {
        return MemoryPressureLevel::Normal;
    }
    let available_percent = u128::from(available_bytes) * 100 / u128::from(total_bytes);
    let swap_percent = u128::from(swap_used_bytes) * 100 / u128::from(total_bytes);
    if available_percent < 8 || swap_percent >= 20 {
        MemoryPressureLevel::Critical
    } else if available_percent < 15 || swap_percent >= 10 {
        MemoryPressureLevel::Warning
    } else {
        MemoryPressureLevel::Normal
    }
}

pub fn capture_snapshot(path: &Path) -> Result<SystemSnapshot, SystemError> {
    SystemMonitor::default().capture(path)
}

fn top_processes(system: &System) -> Vec<ProcessSnapshot> {
    let mut processes = system
        .processes()
        .iter()
        .map(|(pid, process)| {
            let name = process.name().to_string_lossy().into_owned();
            let (protected, protection_reason) = protection(&name);
            ProcessSnapshot {
                pid: pid.as_u32(),
                start_time_unix: process.start_time(),
                name,
                executable: process.exe().and_then(Path::to_str).map(str::to_owned),
                memory_bytes: process.memory(),
                cpu_percent: process.cpu_usage(),
                protected,
                protection_reason,
            }
        })
        .collect::<Vec<_>>();
    processes.sort_by_key(|process| std::cmp::Reverse(process.memory_bytes));
    processes.truncate(12);
    processes
}

fn protection(name: &str) -> (bool, Option<String>) {
    const DEVELOPMENT_NAMES: &[&str] = &[
        "code", "cursor", "codex", "claude", "gemini", "terminal", "iterm", "xcode", "flutter",
        "dart", "gradle", "java", "node", "python", "cargo", "rustc",
    ];
    let lower = name.to_lowercase();
    if DEVELOPMENT_NAMES
        .iter()
        .any(|protected| lower.contains(protected))
    {
        (
            true,
            Some("Developer tool or active job; monitoring is read-only".to_owned()),
        )
    } else {
        (false, None)
    }
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("no mounted disk contains {0}")]
    DiskNotFound(PathBuf),
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use cleaner_domain::MemoryPressureLevel;

    use super::{
        SystemMonitor, capture_snapshot, classify_memory_pressure, parse_vm_stat, protection,
    };

    #[test]
    fn snapshot_reports_bounded_read_only_metrics() -> Result<(), Box<dyn std::error::Error>> {
        let snapshot = capture_snapshot(Path::new("/"))?;
        assert!(snapshot.memory.total_bytes > 0);
        assert!(snapshot.memory.free_bytes <= snapshot.memory.available_bytes);
        assert!(snapshot.memory.available_bytes <= snapshot.memory.total_bytes);
        assert!(snapshot.memory.reclaimable_bytes <= snapshot.memory.available_bytes);
        assert!(snapshot.disk.available_bytes <= snapshot.disk.total_bytes);
        assert!(!snapshot.processes.is_empty());
        Ok(())
    }

    #[test]
    fn developer_jobs_are_explicitly_protected() {
        assert!(protection("Visual Studio Code").0);
        assert!(protection("flutter_tester").0);
        assert!(!protection("Preview").0);
    }

    #[test]
    fn stateful_monitor_can_refresh_repeatedly() -> Result<(), Box<dyn std::error::Error>> {
        let monitor = SystemMonitor::default();
        monitor.capture(Path::new("/"))?;
        monitor.capture(Path::new("/"))?;
        Ok(())
    }

    #[test]
    fn vm_stat_uses_only_reclaimable_pages_for_available_memory()
    -> Result<(), Box<dyn std::error::Error>> {
        let output = "Mach Virtual Memory Statistics: (page size of 16384 bytes)\n\
Pages free:                               4.\n\
Pages active:                           99.\n\
Pages inactive:                          8.\n\
Pages speculative:                       2.\n\
Pages purgeable:                          1.\n\
Pages occupied by compressor:             3.\n\
File-backed pages:                        5.\n";
        let stats = parse_vm_stat(output).ok_or("fixture should parse")?;
        assert_eq!(stats.free, 65_536);
        assert_eq!(stats.available_bytes(), 245_760);
        assert_eq!(stats.compressed, 49_152);
        assert_eq!(stats.file_backed, 81_920);
        Ok(())
    }

    #[test]
    fn pressure_uses_available_capacity_and_swap_without_generating_pressure() {
        const TOTAL: u64 = 20_000_000_000;
        assert_eq!(
            classify_memory_pressure(4_000_000_000, TOTAL, 0),
            MemoryPressureLevel::Normal
        );
        assert_eq!(
            classify_memory_pressure(2_000_000_000, TOTAL, 0),
            MemoryPressureLevel::Warning
        );
        assert_eq!(
            classify_memory_pressure(1_000_000_000, TOTAL, 0),
            MemoryPressureLevel::Critical
        );
        assert_eq!(
            classify_memory_pressure(5_000_000_000, TOTAL, 4_000_000_000),
            MemoryPressureLevel::Critical
        );
    }
}
