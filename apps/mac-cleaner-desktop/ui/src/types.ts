export type SafetyTier = "safe-auto" | "review-required" | "protected";
export type ExecutionMode = "dry_run" | "quarantine";

export interface LoginItemStatus {
  available: boolean;
  enabled: boolean;
}

export interface MemorySnapshot {
  total_bytes: number;
  free_bytes: number;
  available_bytes: number;
  used_bytes: number;
  swap_used_bytes: number;
  reclaimable_bytes: number;
  compressed_bytes: number;
  file_backed_bytes: number;
  pressure: "normal" | "warning" | "critical";
}

export interface DiskSnapshot {
  total_bytes: number;
  available_bytes: number;
}

export interface SystemSnapshot {
  captured_at: string;
  memory: MemorySnapshot;
  disk: DiskSnapshot;
  processes: ProcessSnapshot[];
}

export type StoragePressure = "critical" | "low" | "healthy";
export type StorageDisposition = "reviewable" | "tool-managed" | "protected";
export type StorageOwner = "docker" | "xcode";

export interface StorageHotspot {
  id: string;
  display_name: string;
  path: string;
  bytes: number;
  disposition: StorageDisposition;
  reason: string;
  recommendation: string;
  recovery_cost: string;
}

export interface StorageAnalysis {
  captured_at: string;
  total_bytes: number;
  available_bytes: number;
  pressure: StoragePressure;
  reviewable_bytes: number;
  hotspots: StorageHotspot[];
  warnings: string[];
}

export interface ToolCleanupReceipt {
  plan_id: string;
  kind: "uv-cache-prune" | "simulator-test-recordings";
  target_path: string;
  bytes_before: number;
  bytes_after: number;
  reclaimed_bytes: number;
  completed_at: string;
  summary: string;
}

export interface ProcessSnapshot {
  pid: number;
  start_time_unix: number;
  name: string;
  executable: string | null;
  memory_bytes: number;
  cpu_percent: number;
  protected: boolean;
  protection_reason: string | null;
}

export interface Candidate {
  id: string;
  target_id: string;
  display_name: string;
  lexical_path: string;
  resolved_path: string;
  relative_entry: string;
  category: string;
  reason: string;
  owner: string;
  recovery: string;
  safety_tier: SafetyTier;
  bytes: number;
  files: number;
  last_modified: string;
}

export interface ScanSkip {
  target_id: string;
  path: string;
  reason: string;
}

export interface ScanReport {
  started_at: string;
  completed_at: string;
  candidates: Candidate[];
  skipped: ScanSkip[];
}

export interface CleanupPlan {
  id: string;
  expires_at: string;
  execution_mode: ExecutionMode;
  items: Candidate[];
  total_bytes: number;
  total_files: number;
}

export interface ReceiptItem {
  quarantine_entry_id?: string | null;
  candidate_id: string;
  original_path: string;
  quarantine_path: string | null;
  bytes: number;
  files: number;
  completed_at: string;
}

export type QuarantineAction = "restore" | "purge";

export interface QuarantineEntry {
  id: string;
  plan_id: string;
  candidate_id: string;
  original_path: string;
  quarantine_path: string;
  bytes: number;
  files: number;
  quarantined_at: string;
  state: "active" | "restored" | "purged";
}

export interface ExecutionReceipt {
  plan_id: string;
  mode: ExecutionMode;
  completed_at: string;
  items: ReceiptItem[];
}
