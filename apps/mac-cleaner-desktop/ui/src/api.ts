import { invoke } from "@tauri-apps/api/core";
import type {
  CleanupPlan,
  ExecutionReceipt,
  LoginItemStatus,
  QuarantineAction,
  QuarantineEntry,
  ScanReport,
  StorageAnalysis,
  StorageOwner,
  SystemSnapshot,
  ToolCleanupReceipt,
} from "./types";

export const api = {
  getSnapshot: () => invoke<SystemSnapshot>("get_snapshot"),
  openActivityMonitor: () => invoke<void>("open_activity_monitor"),
  startScan: () => invoke<ScanReport>("start_scan"),
  analyzeStorage: () => invoke<StorageAnalysis>("analyze_storage"),
  openStorageOwner: (owner: StorageOwner) => invoke<void>("open_storage_owner", { owner }),
  confirmUvCachePrune: () => invoke<ToolCleanupReceipt>("confirm_uv_cache_prune"),
  confirmSimulatorRecordingsCleanup: () => invoke<ToolCleanupReceipt>("confirm_simulator_recordings_cleanup"),
  latestUvCleanupReceipt: () => invoke<ToolCleanupReceipt | null>("latest_uv_cleanup_receipt"),
  latestSimulatorCleanupReceipt: () => invoke<ToolCleanupReceipt | null>("latest_simulator_cleanup_receipt"),
  recentToolCleanupReceipts: (limit = 10) =>
    invoke<ToolCleanupReceipt[]>("recent_tool_cleanup_receipts", { limit }),
  latestScan: () => invoke<ScanReport | null>("latest_scan"),
  preparePlan: (ids: string[], dryRun = false) =>
    invoke<CleanupPlan>("prepare_plan", { ids, dryRun }),
  confirmAndExecute: (planId: string) =>
    invoke<ExecutionReceipt>("confirm_and_execute_plan", { planId }),
  recentHistory: (limit = 10) =>
    invoke<ExecutionReceipt[]>("recent_history", { limit }),
  activeQuarantine: (limit = 100) =>
    invoke<QuarantineEntry[]>("active_quarantine", { limit }),
  confirmQuarantineAction: (entryId: string, action: QuarantineAction) =>
    invoke<QuarantineEntry>("confirm_quarantine_action", { entryId, action }),
  launchAtLogin: () => invoke<LoginItemStatus>("launch_at_login"),
  setLaunchAtLogin: (enabled: boolean) =>
    invoke<LoginItemStatus>("set_launch_at_login", { enabled }),
};
