//! Application orchestration shared by the desktop shell and CLI.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use cleaner_domain::{
    ApprovedPlan, ApprovedQuarantineAction, CandidateId, CleanupPlan, ExecutionMode,
    ExecutionReceipt, QuarantineAction, QuarantineActionPlan, QuarantineEntry, ScanReport,
    SystemSnapshot, ToolCleanupPlan,
};
use cleaner_fs::{PlanExecutor, QuarantineExecutor, Scanner, StorageAnalyzer};
use cleaner_policy::{
    approve_plan, approve_quarantine_action, prepare_plan, prepare_quarantine_action,
};
use cleaner_store::Store;
pub use cleaner_store::{LegacyHistoryEntry, MigrationSummary};
use fs2::FileExt;
use parking_lot::{Mutex, RwLock};
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

mod error;
mod fs_safety;
mod tool_cleanup;

pub use error::ServiceError;
use fs_safety::create_private_directory;

const PLAN_TTL: Duration = Duration::from_mins(5);

/// Coordinates scans, immutable approvals, and single-writer cleanup execution.
pub struct AppService {
    home: PathBuf,
    data_dir: PathBuf,
    store: Arc<Store>,
    scanner: Scanner,
    storage_analyzer: StorageAnalyzer,
    executor: PlanExecutor,
    quarantine_executor: QuarantineExecutor,
    monitor: Arc<cleaner_system_macos::SystemMonitor>,
    latest_scan: RwLock<Option<ScanReport>>,
    approvals: Mutex<HashMap<Uuid, ApprovedPlan>>,
    quarantine_approvals: Mutex<HashMap<Uuid, ApprovedQuarantineAction>>,
    tool_cleanup_approvals: Mutex<HashMap<Uuid, ToolCleanupPlan>>,
    scan_gate: AsyncMutex<()>,
    analysis_gate: AsyncMutex<()>,
    mutation_gate: AsyncMutex<()>,
}

impl AppService {
    pub fn open(home: PathBuf, data_dir: PathBuf) -> Result<Self, ServiceError> {
        if home == data_dir || data_dir.starts_with(home.join("Library/Caches")) {
            return Err(ServiceError::UnsafeDataDirectory(data_dir));
        }
        create_private_directory(&data_dir)?;
        let store = Arc::new(Store::open(&data_dir.join("mac-cleaner.sqlite3"))?);
        // Every engine invocation opens the store, including the app's
        // periodic status refresh. Only recover interrupted work when no
        // other process holds the mutation lock; otherwise a long-running
        // cleanup would be marked failed while it is still moving files.
        if let Ok(lock) = try_mutation_lock(&data_dir) {
            store.fail_incomplete_on_startup()?;
            drop(lock);
        }

        Ok(Self {
            scanner: Scanner::new(home.clone()),
            storage_analyzer: StorageAnalyzer::new(home.clone()),
            executor: PlanExecutor::new(home.clone()),
            quarantine_executor: QuarantineExecutor::new(home.clone()),
            monitor: Arc::new(cleaner_system_macos::SystemMonitor::default()),
            home,
            data_dir,
            store,
            latest_scan: RwLock::new(None),
            approvals: Mutex::new(HashMap::new()),
            quarantine_approvals: Mutex::new(HashMap::new()),
            tool_cleanup_approvals: Mutex::new(HashMap::new()),
            scan_gate: AsyncMutex::new(()),
            analysis_gate: AsyncMutex::new(()),
            mutation_gate: AsyncMutex::new(()),
        })
    }

    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }

    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub async fn scan(&self) -> Result<ScanReport, ServiceError> {
        let _guard = self
            .scan_gate
            .try_lock()
            .map_err(|_| ServiceError::ScanInProgress)?;
        let scanner = self.scanner.clone();
        let report = tokio::task::spawn_blocking(move || scanner.scan()).await??;
        *self.latest_scan.write() = Some(report.clone());
        Ok(report)
    }

    #[must_use]
    pub fn latest_scan(&self) -> Option<ScanReport> {
        self.latest_scan.read().clone()
    }

    pub fn prepare_plan(
        &self,
        selected_ids: &[CandidateId],
        execution_mode: ExecutionMode,
    ) -> Result<CleanupPlan, ServiceError> {
        let scan = self
            .latest_scan
            .read()
            .clone()
            .ok_or(ServiceError::ScanRequired)?;
        let plan = prepare_plan(
            &scan.candidates,
            selected_ids,
            Utc::now(),
            PLAN_TTL,
            execution_mode,
        )?;
        self.store.save_plan(&plan)?;
        Ok(plan)
    }

    pub fn approve_plan(&self, plan_id: Uuid) -> Result<(), ServiceError> {
        if self.approvals.lock().contains_key(&plan_id) {
            return Err(ServiceError::AlreadyApproved(plan_id));
        }
        let plan = self
            .store
            .load_plan(plan_id)?
            .ok_or(ServiceError::PlanNotFound(plan_id))?;
        let approved = approve_plan(plan, Utc::now())?;
        self.store
            .mark_approved(plan_id, approved.approval_id(), approved.approved_at())?;
        self.approvals.lock().insert(plan_id, approved);
        Ok(())
    }

    pub fn plan(&self, plan_id: Uuid) -> Result<CleanupPlan, ServiceError> {
        self.store
            .load_plan(plan_id)?
            .ok_or(ServiceError::PlanNotFound(plan_id))
    }

    pub async fn execute_approved(&self, plan_id: Uuid) -> Result<ExecutionReceipt, ServiceError> {
        let _guard = self
            .mutation_gate
            .try_lock()
            .map_err(|_| ServiceError::MutationInProgress)?;
        let approved = self
            .approvals
            .lock()
            .remove(&plan_id)
            .ok_or(ServiceError::ApprovalMissing(plan_id))?;
        let lock = self.acquire_process_lock()?;
        self.store.mark_executing(plan_id)?;
        let executor = self.executor.clone();
        let result = tokio::task::spawn_blocking(move || {
            let _lock = lock;
            executor.execute(approved)
        })
        .await;

        match result {
            Ok(Ok(receipt)) => {
                self.store.save_receipt(&receipt)?;
                Ok(receipt)
            }
            Ok(Err(error)) => {
                self.store.mark_failed(plan_id, &error.to_string())?;
                Err(ServiceError::Filesystem(error))
            }
            Err(error) => {
                self.store.mark_failed(plan_id, &error.to_string())?;
                Err(ServiceError::TaskJoin(error))
            }
        }
    }

    pub async fn system_snapshot(&self) -> Result<SystemSnapshot, ServiceError> {
        let home = self.home.clone();
        let monitor = Arc::clone(&self.monitor);
        Ok(tokio::task::spawn_blocking(move || monitor.capture(&home)).await??)
    }

    pub fn recent_history(&self, limit: u32) -> Result<Vec<ExecutionReceipt>, ServiceError> {
        Ok(self.store.recent_receipts(limit.min(100))?)
    }

    pub fn active_quarantine(&self, limit: u32) -> Result<Vec<QuarantineEntry>, ServiceError> {
        Ok(self.store.active_quarantine(limit.min(100))?)
    }

    pub fn prepare_quarantine_action(
        &self,
        entry_id: Uuid,
        action: QuarantineAction,
    ) -> Result<QuarantineActionPlan, ServiceError> {
        let entry = self.store.active_quarantine_entry(entry_id)?;
        let plan = prepare_quarantine_action(entry, action, Utc::now(), PLAN_TTL)?;
        self.store.save_quarantine_action(&plan)?;
        Ok(plan)
    }

    pub fn quarantine_action_plan(
        &self,
        action_id: Uuid,
    ) -> Result<QuarantineActionPlan, ServiceError> {
        Ok(self.store.load_quarantine_action(action_id)?)
    }

    pub fn approve_quarantine_action(&self, action_id: Uuid) -> Result<(), ServiceError> {
        if self.quarantine_approvals.lock().contains_key(&action_id) {
            return Err(ServiceError::AlreadyApproved(action_id));
        }
        let plan = self.store.load_quarantine_action(action_id)?;
        let approved = approve_quarantine_action(plan, Utc::now())?;
        self.store.mark_quarantine_action_approved(
            action_id,
            approved.approval_id(),
            approved.approved_at(),
        )?;
        self.quarantine_approvals.lock().insert(action_id, approved);
        Ok(())
    }

    pub async fn execute_quarantine_action(
        &self,
        action_id: Uuid,
    ) -> Result<QuarantineEntry, ServiceError> {
        let _guard = self
            .mutation_gate
            .try_lock()
            .map_err(|_| ServiceError::MutationInProgress)?;
        let approved = self
            .quarantine_approvals
            .lock()
            .remove(&action_id)
            .ok_or(ServiceError::ApprovalMissing(action_id))?;
        let lock = self.acquire_process_lock()?;
        self.store.mark_quarantine_action_executing(action_id)?;
        let executor = self.quarantine_executor.clone();
        let result = tokio::task::spawn_blocking(move || {
            let _lock = lock;
            executor.execute(approved)
        })
        .await;
        match result {
            Ok(Ok(entry)) => {
                self.store.complete_quarantine_action(action_id, &entry)?;
                Ok(entry)
            }
            Ok(Err(error)) => {
                self.store
                    .fail_quarantine_action(action_id, &error.to_string())?;
                Err(ServiceError::Filesystem(error))
            }
            Err(error) => {
                self.store
                    .fail_quarantine_action(action_id, &error.to_string())?;
                Err(ServiceError::TaskJoin(error))
            }
        }
    }

    pub fn import_legacy(&self, source_root: &Path) -> Result<MigrationSummary, ServiceError> {
        Ok(self.store.import_legacy(source_root)?)
    }

    pub fn legacy_history(&self, limit: u32) -> Result<Vec<LegacyHistoryEntry>, ServiceError> {
        Ok(self.store.legacy_history(limit)?)
    }

    fn acquire_process_lock(&self) -> Result<File, ServiceError> {
        try_mutation_lock(&self.data_dir)
    }
}

fn try_mutation_lock(data_dir: &Path) -> Result<File, ServiceError> {
    let path = data_dir.join("mutation.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|source| ServiceError::LockFile {
            path: path.clone(),
            source,
        })?;
    file.try_lock_exclusive()
        .map_err(|_| ServiceError::MutationInProgress)?;
    Ok(file)
}
