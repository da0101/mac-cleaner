import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { errorMessage, formatBytes } from "../format";
import type {
  CleanupPlan,
  ExecutionReceipt,
  QuarantineAction,
  QuarantineEntry,
  ScanReport,
  StorageAnalysis,
  StorageOwner,
  SystemSnapshot,
  ToolCleanupReceipt,
} from "../types";

export type Phase = "idle" | "scanning" | "planning" | "executing";
export type ToolOperation = "uv" | "simulator" | null;

export function useMacCleaner() {
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null);
  const [scan, setScan] = useState<ScanReport | null>(null);
  const [storage, setStorage] = useState<StorageAnalysis | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [plan, setPlan] = useState<CleanupPlan | null>(null);
  const [history, setHistory] = useState<ExecutionReceipt[]>([]);
  const [toolHistory, setToolHistory] = useState<ToolCleanupReceipt[]>([]);
  const [quarantine, setQuarantine] = useState<QuarantineEntry[]>([]);
  const [launchAtLogin, setLaunchAtLogin] = useState(false);
  const [loginItemAvailable, setLoginItemAvailable] = useState(false);
  const [phase, setPhase] = useState<Phase>("idle");
  const [storageAnalyzing, setStorageAnalyzing] = useState(false);
  const [toolOperation, setToolOperation] = useState<ToolOperation>(null);
  const [uvStatus, setUvStatus] = useState<string | null>(null);
  const [simulatorStatus, setSimulatorStatus] = useState<string | null>(null);
  const [notice, setNotice] = useState("Ready. No automatic storage cleanup is enabled.");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    Promise.all([
      api.getSnapshot(),
      api.latestScan(),
      api.recentHistory(),
      api.recentToolCleanupReceipts(),
      api.activeQuarantine(),
      api.launchAtLogin(),
    ])
      .then(([nextSnapshot, latest, receipts, toolReceipts, entries, launches]) => {
        if (!active) return;
        setSnapshot(nextSnapshot);
        setScan(latest);
        setHistory(receipts);
        setToolHistory(toolReceipts);
        setQuarantine(entries);
        setLaunchAtLogin(launches.enabled);
        setLoginItemAvailable(launches.available);
        const latestUv = toolReceipts.find((receipt) => receipt.kind === "uv-cache-prune");
        const latestSimulator = toolReceipts.find((receipt) => receipt.kind === "simulator-test-recordings");
        if (latestUv) {
          setUvStatus(latestUv.reclaimed_bytes > 0
            ? `Last completed prune reclaimed ${formatBytes(latestUv.reclaimed_bytes)} from ${latestUv.target_path}.`
            : "Last completed prune found no more unused uv cache entries.");
        }
        if (latestSimulator) {
          setSimulatorStatus(`Last cleanup removed ${formatBytes(latestSimulator.reclaimed_bytes)} of old simulator test recordings.`);
        }
      })
      .catch((reason) => active && setError(errorMessage(reason)));

    const listeners = Promise.all([
      listen<SystemSnapshot>("system-snapshot", ({ payload }) => setSnapshot(payload)),
      listen<ScanReport>("scan-complete", ({ payload }) => {
        setScan(payload);
        setSelected(new Set());
        setPlan(null);
        setNotice(`Scan complete. ${payload.candidates.length} reviewable item(s) found.`);
      }),
      listen<string>("scan-error", ({ payload }) => {
        setError(payload);
      }),
      listen<string>("monitor-error", ({ payload }) => setError(payload)),
    ]);
    return () => {
      active = false;
      void listeners.then((unlisten) => unlisten.forEach((stop) => stop()));
    };
  }, []);

  const selectedBytes = useMemo(
    () => scan?.candidates
      .filter((candidate) => selected.has(candidate.id))
      .reduce((total, candidate) => total + candidate.bytes, 0) ?? 0,
    [scan, selected],
  );

  async function refreshStorageState() {
    try {
      setSnapshot(await api.getSnapshot());
      setStorage(await api.analyzeStorage());
    } catch (reason) {
      setError(`Cleanup completed, but storage refresh failed: ${errorMessage(reason)}`);
    }
  }

  function recordToolReceipt(receipt: ToolCleanupReceipt) {
    setToolHistory((current) => [receipt, ...current.filter((item) => item.plan_id !== receipt.plan_id)]
      .sort((left, right) => Date.parse(right.completed_at) - Date.parse(left.completed_at))
      .slice(0, 10));
  }

  async function runScan() {
    setPhase("scanning");
    setStorageAnalyzing(true);
    setError(null);
    setPlan(null);
    setSelected(new Set());
    setNotice("Analyzing storage and scanning exact approved cleanup locations…");
    try {
      const [nextScan, nextStorage] = await Promise.all([api.startScan(), api.analyzeStorage()]);
      setScan(nextScan);
      setStorage(nextStorage);
      setNotice(`Analysis complete. ${nextStorage.hotspots.length} known storage location(s) and ${nextScan.candidates.length} direct cleanup candidate(s) found.`);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setPhase("idle");
      setStorageAnalyzing(false);
    }
  }

  async function analyzeStorage() {
    setStorageAnalyzing(true);
    setError(null);
    setNotice("Measuring known storage hotspots from metadata and trusted developer-cache signatures…");
    try {
      const report = await api.analyzeStorage();
      setStorage(report);
      setNotice(`Storage analysis complete. ${report.hotspots.length} large known locations explained.`);
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setStorageAnalyzing(false);
    }
  }

  async function reviewCleanup() {
    await runScan();
    window.setTimeout(() => {
      document.getElementById("storage-title")?.scrollIntoView({ behavior: "smooth", block: "start" });
    }, 100);
  }

  async function openStorageOwner(owner: StorageOwner) {
    setError(null);
    try {
      await api.openStorageOwner(owner);
      setNotice(owner === "docker"
        ? "Opened Docker Desktop. Review reclaimable data without deleting volumes."
        : "Opened Xcode. Review shutdown simulator devices before deletion.");
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  async function pruneUvCache() {
    setPhase("executing");
    setToolOperation("uv");
    setError(null);
    setUvStatus("Waiting for confirmation. Nothing has been removed yet.");
    setNotice("Preparing an exact uv cache plan…");
    try {
      const receipt = await api.confirmUvCachePrune();
      const result = receipt.reclaimed_bytes > 0
        ? `Completed: uv reclaimed ${formatBytes(receipt.reclaimed_bytes)} from ${receipt.target_path}.`
        : "Completed: uv found no more unused cache entries.";
      setUvStatus(result);
      setNotice(result);
      recordToolReceipt(receipt);
      setToolOperation(null);
      setPhase("idle");
      void refreshStorageState();
    } catch (reason) {
      const message = errorMessage(reason);
      if (message.startsWith("uv cache cleanup cancelled")) {
        setUvStatus("Cancelled: uv cache was not changed.");
        setNotice(message);
      } else {
        setUvStatus(`Cleanup failed: ${message}`);
        setError(message);
      }
    } finally {
      setToolOperation(null);
      setPhase("idle");
    }
  }

  async function cleanSimulatorRecordings() {
    setPhase("executing");
    setToolOperation("simulator");
    setError(null);
    setSimulatorStatus("Waiting for confirmation. Nothing has been removed yet.");
    setNotice("Preparing an exact simulator recording plan…");
    try {
      const receipt = await api.confirmSimulatorRecordingsCleanup();
      const result = `Completed: removed ${formatBytes(receipt.reclaimed_bytes)} of old simulator test recordings.`;
      setSimulatorStatus(result);
      setNotice(result);
      recordToolReceipt(receipt);
      setStorage((current) => current ? {
        ...current,
        reviewable_bytes: Math.max(0, current.reviewable_bytes - receipt.reclaimed_bytes),
        hotspots: current.hotspots.filter((hotspot) => hotspot.display_name !== "Simulator test recordings"),
      } : current);
      setToolOperation(null);
      setPhase("idle");
      void refreshStorageState();
    } catch (reason) {
      const message = errorMessage(reason);
      if (message.startsWith("Simulator recording cleanup cancelled")) {
        setSimulatorStatus("Cancelled: simulator recordings were not changed.");
        setNotice(message);
      } else {
        setSimulatorStatus(`Cleanup failed: ${message}`);
        setError(message);
      }
    } finally {
      setToolOperation(null);
      setPhase("idle");
    }
  }

  function toggleCandidate(id: string) {
    setSelected((current) => {
      const next = new Set(current);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
    setPlan(null);
  }

  async function reviewPlan() {
    if (selected.size === 0) return;
    setPhase("planning");
    setError(null);
    try {
      setPlan(await api.preparePlan([...selected]));
      setNotice("Plan prepared. Review every path before native confirmation.");
    } catch (reason) {
      setError(errorMessage(reason));
    } finally {
      setPhase("idle");
    }
  }

  async function executePlan() {
    if (!plan) return;
    setPhase("executing");
    setError(null);
    try {
      const receipt = await api.confirmAndExecute(plan.id);
      setNotice(`${receipt.items.length} item(s) moved to recoverable quarantine.`);
      setHistory(await api.recentHistory());
      setQuarantine(await api.activeQuarantine());
      setScan(await api.startScan());
      setSelected(new Set());
      setPlan(null);
    } catch (reason) {
      const message = errorMessage(reason);
      if (message.startsWith("Cleanup cancelled")) setNotice(message);
      else setError(message);
    } finally {
      setPhase("idle");
    }
  }

  async function changeLaunchAtLogin(enabled: boolean) {
    setError(null);
    try {
      const status = await api.setLaunchAtLogin(enabled);
      setLaunchAtLogin(status.enabled);
      setLoginItemAvailable(status.available);
      setNotice(enabled ? "Launch at login enabled." : "Launch at login disabled.");
    } catch (reason) {
      setError(errorMessage(reason));
    }
  }

  async function runQuarantineAction(entryId: string, action: QuarantineAction) {
    setPhase("executing");
    setError(null);
    try {
      await api.confirmQuarantineAction(entryId, action);
      setQuarantine(await api.activeQuarantine());
      setNotice(action === "restore"
        ? "Item restored to its original path."
        : "Item permanently deleted; disk space reclaimed.");
    } catch (reason) {
      const message = errorMessage(reason);
      if (message.startsWith("Quarantine action cancelled")) setNotice(message);
      else setError(message);
    } finally {
      setPhase("idle");
    }
  }

  return {
    snapshot,
    scan,
    storage,
    selected,
    plan,
    history,
    toolHistory,
    quarantine,
    launchAtLogin,
    loginItemAvailable,
    phase,
    storageAnalyzing,
    toolOperation,
    uvStatus,
    simulatorStatus,
    notice,
    error,
    selectedBytes,
    runScan,
    analyzeStorage,
    reviewCleanup,
    openStorageOwner,
    pruneUvCache,
    cleanSimulatorRecordings,
    toggleCandidate,
    reviewPlan,
    executePlan,
    changeLaunchAtLogin,
    runQuarantineAction,
  };
}
