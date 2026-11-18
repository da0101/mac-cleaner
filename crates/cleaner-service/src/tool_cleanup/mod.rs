mod simulator;
mod uv;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use cleaner_domain::{
    StorageAnalysis, StorageCategory, StorageComparisonKind, StorageComparisonState,
    ToolCleanupKind, ToolCleanupOperation, ToolCleanupPlan, ToolCleanupReceipt, ToolCleanupState,
};
use uuid::Uuid;

use crate::fs_safety::{directory_allocated_bytes, directory_identity};
use crate::{AppService, ServiceError};

const SIMULATOR_RECORDING_MIN_AGE: Duration = Duration::from_hours(72);

impl AppService {
    pub fn latest_storage_analysis(&self) -> Result<Option<StorageAnalysis>, ServiceError> {
        Ok(self.store.latest_storage_snapshot()?)
    }

    pub async fn analyze_storage(&self) -> Result<StorageAnalysis, ServiceError> {
        let _guard = self
            .analysis_gate
            .try_lock()
            .map_err(|_| ServiceError::AnalysisInProgress)?;
        let snapshot = self.system_snapshot().await?;
        let analyzer = self.storage_analyzer.clone();
        let mut report = tokio::task::spawn_blocking(move || {
            analyzer.analyze(snapshot.disk.total_bytes, snapshot.disk.available_bytes)
        })
        .await??;
        let home = self.home.clone();
        if let Ok(target) = tokio::task::spawn_blocking(move || {
            simulator::discover(&home, SIMULATOR_RECORDING_MIN_AGE)
        })
        .await?
        {
            report.reviewable_bytes = report.reviewable_bytes.saturating_add(target.bytes);
            report.hotspots.push(cleaner_domain::StorageHotspot {
                id: format!("simulator-test-recordings:{}", target.device_udid),
                display_name: "Simulator test recordings".to_owned(),
                path: target.path,
                bytes: target.bytes,
                category: StorageCategory::TestArtifacts,
                disposition: cleaner_domain::StorageDisposition::Reviewable,
                comparison_state: StorageComparisonState::NoBaseline,
                previous_bytes: None,
                delta_bytes: None,
                reason: format!(
                    "{} old QuickTime test recordings from {}. App data and the simulator itself are outside this target.",
                    target.files.len(), target.device_name
                ),
                recommendation: if target.device_was_booted {
                    "Review the exact recordings. After confirmation, Mac Cleaner will shut down only this simulator and remove the verified old video files.".to_owned()
                } else {
                    "Review the exact recordings, then remove only the verified old video files."
                        .to_owned()
                },
                recovery_cost:
                    "Deleted test recordings cannot be restored; rerun the tests to create new results."
                        .to_owned(),
            });
            report
                .hotspots
                .sort_by_key(|item| std::cmp::Reverse(item.bytes));
        }
        // Swap files live on a separate macOS volume, so the folder scan cannot see them,
        // yet they take real disk space. Measured from live memory data and shown read-only.
        if snapshot.memory.swap_used_bytes > 0 {
            report.hotspots.push(cleaner_domain::StorageHotspot {
                id: "system-memory-swap".to_owned(),
                display_name: "Memory swap files".to_owned(),
                path: std::path::PathBuf::from("/System/Volumes/VM"),
                bytes: snapshot.memory.swap_used_bytes,
                category: StorageCategory::SystemMemory,
                disposition: cleaner_domain::StorageDisposition::Protected,
                comparison_state: StorageComparisonState::NoBaseline,
                previous_bytes: None,
                delta_bytes: None,
                reason: "macOS writes memory to disk when RAM runs short. It grows with memory pressure."
                    .to_owned(),
                recommendation:
                    "Not deleted by hand. It shrinks after you quit heavy apps and restart them or reboot."
                        .to_owned(),
                recovery_cost: "Managed by macOS; no action in Mac Cleaner.".to_owned(),
            });
            report
                .hotspots
                .sort_by_key(|item| std::cmp::Reverse(item.bytes));
        }
        let comparison = if let Some(baseline) = self.store.storage_baseline()? {
            Some((baseline, StorageComparisonKind::Baseline))
        } else {
            self.store
                .latest_storage_snapshot()?
                .map(|snapshot| (snapshot, StorageComparisonKind::Previous))
        };
        report.compare_with(
            comparison
                .as_ref()
                .map(|(snapshot, kind)| (snapshot, *kind)),
        );
        self.store.save_storage_snapshot(&report)?;
        Ok(report)
    }

    pub fn set_storage_baseline(&self, snapshot_id: Uuid) -> Result<(), ServiceError> {
        Ok(self.store.set_storage_baseline(snapshot_id)?)
    }

    pub async fn prepare_uv_cache_prune(&self) -> Result<ToolCleanupPlan, ServiceError> {
        let target_path = self.home.join(".cache/uv");
        let executable_path = uv::find_executable()?;
        let target_for_measurement = target_path.clone();
        let (target_identity, bytes_before) = tokio::task::spawn_blocking(move || {
            let identity = directory_identity(&target_for_measurement)?;
            let bytes = directory_allocated_bytes(&target_for_measurement, identity.device)?;
            Ok::<_, ServiceError>((identity, bytes))
        })
        .await??;
        let created_at = Utc::now();
        let plan = ToolCleanupPlan {
            id: Uuid::new_v4(),
            kind: ToolCleanupKind::UvCachePrune,
            state: ToolCleanupState::Prepared,
            target_path: target_path.clone(),
            target_identity,
            operation: ToolCleanupOperation::Command {
                executable_path,
                arguments: vec![
                    "cache".to_owned(),
                    "prune".to_owned(),
                    "--cache-dir".to_owned(),
                    target_path.to_string_lossy().into_owned(),
                    "--color".to_owned(),
                    "never".to_owned(),
                    "--no-progress".to_owned(),
                    "--no-config".to_owned(),
                ],
            },
            bytes_before,
            created_at,
            expires_at: created_at + chrono::Duration::minutes(5),
        };
        self.store.save_tool_cleanup_plan(&plan)?;
        Ok(plan)
    }

    pub async fn prepare_simulator_recordings_cleanup(
        &self,
    ) -> Result<ToolCleanupPlan, ServiceError> {
        let home = self.home.clone();
        let target = tokio::task::spawn_blocking(move || {
            simulator::discover(&home, SIMULATOR_RECORDING_MIN_AGE)
        })
        .await??;
        let created_at = Utc::now();
        let plan = ToolCleanupPlan {
            id: Uuid::new_v4(),
            kind: ToolCleanupKind::SimulatorTestRecordings,
            state: ToolCleanupState::Prepared,
            target_path: target.path,
            target_identity: target.identity,
            operation: ToolCleanupOperation::SimulatorAttachments {
                device_udid: target.device_udid,
                device_name: target.device_name,
                device_was_booted: target.device_was_booted,
                files: target.files,
            },
            bytes_before: target.bytes,
            created_at,
            expires_at: created_at + chrono::Duration::minutes(5),
        };
        self.store.save_tool_cleanup_plan(&plan)?;
        Ok(plan)
    }

    pub fn approve_tool_cleanup(&self, plan_id: Uuid) -> Result<(), ServiceError> {
        if self.tool_cleanup_approvals.lock().contains_key(&plan_id) {
            return Err(ServiceError::AlreadyApproved(plan_id));
        }
        let plan = self.store.load_tool_cleanup_plan(plan_id)?;
        validate_plan(&self.home, &plan)?;
        if plan.expires_at <= Utc::now() {
            return Err(ServiceError::ToolCleanupExpired(plan_id));
        }
        let approved_at = Utc::now();
        self.store
            .mark_tool_cleanup_approved(plan_id, Uuid::new_v4(), approved_at)?;
        self.tool_cleanup_approvals.lock().insert(plan_id, plan);
        Ok(())
    }

    pub fn tool_cleanup_plan(&self, plan_id: Uuid) -> Result<ToolCleanupPlan, ServiceError> {
        Ok(self.store.load_tool_cleanup_plan(plan_id)?)
    }

    pub async fn execute_tool_cleanup(
        &self,
        plan_id: Uuid,
    ) -> Result<ToolCleanupReceipt, ServiceError> {
        let _guard = self
            .mutation_gate
            .try_lock()
            .map_err(|_| ServiceError::MutationInProgress)?;
        let plan = self
            .tool_cleanup_approvals
            .lock()
            .remove(&plan_id)
            .ok_or(ServiceError::ApprovalMissing(plan_id))?;
        validate_plan(&self.home, &plan)?;
        let lock = self.acquire_process_lock()?;
        self.store.mark_tool_cleanup_executing(plan_id)?;
        let home = self.home.clone();
        let store = Arc::clone(&self.store);
        let result = tokio::task::spawn_blocking(move || {
            let _lock = lock;
            match plan.kind {
                ToolCleanupKind::UvCachePrune => uv::execute(plan_id, plan, &home),
                ToolCleanupKind::SimulatorTestRecordings => {
                    simulator::execute(plan_id, plan, &store)
                }
            }
        })
        .await;

        match result {
            Ok(Ok(receipt)) => {
                self.store.save_tool_cleanup_receipt(&receipt)?;
                Ok(receipt)
            }
            Ok(Err(error)) => {
                self.store.fail_tool_cleanup(plan_id, &error.to_string())?;
                Err(error)
            }
            Err(error) => {
                self.store.fail_tool_cleanup(plan_id, &error.to_string())?;
                Err(ServiceError::TaskJoin(error))
            }
        }
    }

    pub fn latest_uv_cleanup_receipt(&self) -> Result<Option<ToolCleanupReceipt>, ServiceError> {
        Ok(self
            .store
            .latest_tool_cleanup_receipt(ToolCleanupKind::UvCachePrune)?)
    }

    pub fn latest_simulator_cleanup_receipt(
        &self,
    ) -> Result<Option<ToolCleanupReceipt>, ServiceError> {
        Ok(self
            .store
            .latest_tool_cleanup_receipt(ToolCleanupKind::SimulatorTestRecordings)?)
    }

    pub fn recent_tool_cleanup_receipts(
        &self,
        limit: u32,
    ) -> Result<Vec<ToolCleanupReceipt>, ServiceError> {
        Ok(self.store.recent_tool_cleanup_receipts(limit.min(100))?)
    }
}

fn validate_plan(home: &Path, plan: &ToolCleanupPlan) -> Result<(), ServiceError> {
    match plan.kind {
        ToolCleanupKind::UvCachePrune => uv::validate(home, plan),
        ToolCleanupKind::SimulatorTestRecordings => simulator::validate(home, plan),
    }
}
