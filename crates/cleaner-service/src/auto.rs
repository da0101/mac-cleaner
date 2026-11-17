//! Auto-clean orchestration: the owner's opt-in settings, the RAM policy
//! tick, and scheduled removal of opted-in cleanup categories.
//!
//! Storage removal reuses the ordinary pipeline (catalog discovery, exact plan,
//! approval, identity revalidation, quarantine, purge, receipts). Nothing here
//! signals a process: the native app asks the chosen app to quit.

use chrono::{DateTime, Duration, Utc};
use cleaner_domain::{
    AutoCleanSettings, AutoLogEntry, AutoLogKind, AutoRamDecision, AutoRamState, AutoStorageReport,
    AutoStorageSettings, AutoStorageState, AutoTickReport, CandidateId, ExecutionMode,
    QuarantineAction,
};
use cleaner_policy::{plan_ram_relief, validate_auto_settings};

use crate::{AppService, ServiceError};

const SETTINGS_KEY: &str = "auto_clean_settings";
const RAM_STATE_KEY: &str = "auto_ram_state";
const STORAGE_STATE_KEY: &str = "auto_storage_state";
const LOG_KEY: &str = "auto_clean_log";
const LOG_CAP: usize = 100;
/// A plan holds at most 100 candidates; leftovers wait for the next run.
const MAX_ITEMS_PER_RUN: usize = 100;

impl AppService {
    /// Lower-case names of apps that can never be on the quit list.
    #[must_use]
    pub fn auto_protected_apps(&self) -> Vec<String> {
        cleaner_policy::never_quit_names()
    }

    pub fn auto_settings(&self) -> Result<AutoCleanSettings, ServiceError> {
        Ok(self.store.get_setting(SETTINGS_KEY)?.unwrap_or_default())
    }

    /// Validate, clamp, and save the owner's auto-clean settings.
    pub fn set_auto_settings(
        &self,
        settings: AutoCleanSettings,
    ) -> Result<AutoCleanSettings, ServiceError> {
        let clean = validate_auto_settings(settings)?;
        self.store.set_setting(SETTINGS_KEY, &clean)?;
        Ok(clean)
    }

    /// Newest first.
    pub fn auto_log(&self, limit: u32) -> Result<Vec<AutoLogEntry>, ServiceError> {
        let mut log: Vec<AutoLogEntry> = self.store.get_setting(LOG_KEY)?.unwrap_or_default();
        log.reverse();
        log.truncate(usize::try_from(limit).unwrap_or(LOG_CAP).min(LOG_CAP));
        Ok(log)
    }

    fn push_log(&self, kind: AutoLogKind, summary: String, bytes: u64) -> Result<(), ServiceError> {
        let mut log: Vec<AutoLogEntry> = self.store.get_setting(LOG_KEY)?.unwrap_or_default();
        log.push(AutoLogEntry {
            at: Utc::now(),
            kind,
            summary,
            bytes,
        });
        if log.len() > LOG_CAP {
            log.drain(..log.len() - LOG_CAP);
        }
        self.store.set_setting(LOG_KEY, &log)?;
        Ok(())
    }

    /// One scheduler step: decide on RAM relief and run any due storage clean.
    pub async fn auto_tick(&self) -> Result<AutoTickReport, ServiceError> {
        let settings = self.auto_settings()?;
        if !settings.ram.enabled && !settings.storage.enabled {
            return Ok(AutoTickReport {
                ram: AutoRamDecision::Disabled,
                storage: None,
            });
        }
        let snapshot = self.system_snapshot().await?;
        let now = Utc::now();

        let ram = if settings.ram.enabled {
            let monitor = std::sync::Arc::clone(&self.monitor);
            let processes = tokio::task::spawn_blocking(move || monitor.all_processes()).await?;
            let mut state: AutoRamState =
                self.store.get_setting(RAM_STATE_KEY)?.unwrap_or_default();
            let decision =
                plan_ram_relief(&settings.ram, &snapshot.memory, &processes, &mut state, now);
            self.store.set_setting(RAM_STATE_KEY, &state)?;
            decision
        } else {
            AutoRamDecision::Disabled
        };

        let mut storage = None;
        if settings.storage.enabled && !settings.storage.target_ids.is_empty() {
            let mut state: AutoStorageState = self
                .store
                .get_setting(STORAGE_STATE_KEY)?
                .unwrap_or_default();
            if storage_is_due(
                &settings.storage,
                &state,
                snapshot.disk.available_bytes,
                now,
            ) {
                match self.run_auto_storage(&settings.storage, now).await {
                    Ok(report) => {
                        state.last_run = Some(now);
                        self.store.set_setting(STORAGE_STATE_KEY, &state)?;
                        if report.items > 0 {
                            self.push_log(
                                AutoLogKind::StorageClean,
                                format!(
                                    "Removed {} item(s) from your approved categories",
                                    report.items
                                ),
                                report.bytes_freed,
                            )?;
                        }
                        storage = Some(report);
                    }
                    // Another cleanup or scan is running; try again next tick.
                    Err(
                        ServiceError::MutationInProgress
                        | ServiceError::ScanInProgress
                        | ServiceError::AnalysisInProgress,
                    ) => {}
                    Err(error) => {
                        state.last_run = Some(now);
                        self.store.set_setting(STORAGE_STATE_KEY, &state)?;
                        self.push_log(
                            AutoLogKind::Error,
                            format!("Scheduled cleanup stopped: {error}"),
                            0,
                        )?;
                    }
                }
            }
        }
        Ok(AutoTickReport { ram, storage })
    }

    /// Record that the native app asked `app_name` to quit.
    pub fn auto_report_quit(&self, app_name: &str, bytes: u64) -> Result<(), ServiceError> {
        let now = Utc::now();
        let mut state: AutoRamState = self.store.get_setting(RAM_STATE_KEY)?.unwrap_or_default();
        state.last_quit_at = Some(now);
        state.recent_quits.push((app_name.to_owned(), now));
        self.store.set_setting(RAM_STATE_KEY, &state)?;
        self.push_log(
            AutoLogKind::RamQuit,
            format!(
                "Asked {app_name} to quit; it was using about {}",
                human_bytes(bytes)
            ),
            bytes,
        )
    }

    async fn run_auto_storage(
        &self,
        settings: &AutoStorageSettings,
        now: DateTime<Utc>,
    ) -> Result<AutoStorageReport, ServiceError> {
        let report = self.scan().await?;
        let cutoff = now - Duration::hours(i64::try_from(settings.min_idle_hours).unwrap_or(24));
        let ids: Vec<CandidateId> = report
            .candidates
            .iter()
            .filter(|candidate| {
                settings.target_ids.contains(&candidate.target_id.0)
                    && candidate.last_modified <= cutoff
            })
            .take(MAX_ITEMS_PER_RUN)
            .map(|candidate| candidate.id.clone())
            .collect();
        let skipped = (report.candidates.len() - ids.len()) as u64;
        if ids.is_empty() {
            return Ok(AutoStorageReport {
                items: 0,
                bytes_freed: 0,
                skipped,
            });
        }

        let plan = self.prepare_plan(&ids, ExecutionMode::Quarantine)?;
        self.approve_plan(plan.id)?;
        let receipt = self.execute_approved(plan.id).await?;

        let mut bytes_freed = 0_u64;
        let mut items = 0_u64;
        for entry in self
            .active_quarantine(100)?
            .into_iter()
            .filter(|entry| entry.plan_id == receipt.plan_id)
        {
            let action = self.prepare_quarantine_action(entry.id, QuarantineAction::Purge)?;
            self.approve_quarantine_action(action.id)?;
            let purged = self.execute_quarantine_action(action.id).await?;
            bytes_freed = bytes_freed.saturating_add(purged.bytes);
            items += 1;
        }
        Ok(AutoStorageReport {
            items,
            bytes_freed,
            skipped,
        })
    }
}

fn storage_is_due(
    settings: &AutoStorageSettings,
    state: &AutoStorageState,
    disk_available_bytes: u64,
    now: DateTime<Utc>,
) -> bool {
    let Some(last) = state.last_run else {
        return true;
    };
    let elapsed = now - last;
    let interval = Duration::hours(i64::try_from(settings.interval_hours).unwrap_or(6));
    elapsed >= interval
        || (disk_available_bytes < settings.low_free_bytes && elapsed >= Duration::hours(1))
}

fn human_bytes(bytes: u64) -> String {
    const MIB: u64 = 1_048_576;
    const GIB: u64 = 1_073_741_824;
    if bytes >= GIB {
        format!("{}.{} GB", bytes / GIB, (bytes % GIB) * 10 / GIB)
    } else {
        format!("{} MB", bytes / MIB)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use cleaner_domain::{AutoStorageSettings, AutoStorageState, GIB};

    use super::{human_bytes, storage_is_due};

    #[test]
    fn storage_runs_first_time_then_on_interval_or_when_disk_is_low() {
        let settings = AutoStorageSettings {
            enabled: true,
            interval_hours: 6,
            low_free_bytes: 40 * GIB,
            min_idle_hours: 24,
            target_ids: vec!["npm-npx-cache".to_owned()],
        };
        let now = Utc
            .timestamp_opt(1_800_000_000, 0)
            .single()
            .unwrap_or_default();
        let plenty = 100 * GIB;
        let low = 10 * GIB;

        assert!(storage_is_due(
            &settings,
            &AutoStorageState::default(),
            plenty,
            now
        ));
        let ran_recently = AutoStorageState {
            last_run: Some(now - Duration::minutes(30)),
        };
        assert!(!storage_is_due(&settings, &ran_recently, plenty, now));
        // Low disk does not allow more than one run an hour.
        assert!(!storage_is_due(&settings, &ran_recently, low, now));
        let ran_two_hours_ago = AutoStorageState {
            last_run: Some(now - Duration::hours(2)),
        };
        assert!(storage_is_due(&settings, &ran_two_hours_ago, low, now));
        assert!(!storage_is_due(&settings, &ran_two_hours_ago, plenty, now));
        let ran_yesterday = AutoStorageState {
            last_run: Some(now - Duration::hours(7)),
        };
        assert!(storage_is_due(&settings, &ran_yesterday, plenty, now));
    }

    #[test]
    fn byte_sizes_read_naturally() {
        assert_eq!(human_bytes(1_600 * 1_048_576), "1.5 GB");
        assert_eq!(human_bytes(300 * 1_048_576), "300 MB");
    }
}
