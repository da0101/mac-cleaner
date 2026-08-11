import { AlertTriangle, Database, ExternalLink, HardDrive, LoaderCircle, LockKeyhole, Search, Wrench } from "lucide-react";
import { useState } from "react";
import { formatBytes } from "./format";
import type { ToolOperation } from "./hooks/useMacCleaner";
import type { StorageAnalysis, StorageDisposition, StorageOwner, SystemSnapshot } from "./types";

interface Props {
  snapshot: SystemSnapshot | null;
  storage: StorageAnalysis | null;
  analyzing: boolean;
  scanning: boolean;
  onAnalyze: () => Promise<void>;
  onReviewCleanup: () => Promise<void>;
  onOpenOwner: (owner: StorageOwner) => Promise<void>;
  onPruneUv: () => Promise<void>;
  onCleanSimulatorRecordings: () => Promise<void>;
  toolOperation: ToolOperation;
  uvStatus: string | null;
  simulatorStatus: string | null;
}

export function StorageSection({ snapshot, storage, analyzing, scanning, onAnalyze, onReviewCleanup, onOpenOwner, onPruneUv, onCleanSimulatorRecordings, toolOperation, uvStatus, simulatorStatus }: Props) {
  const [showAll, setShowAll] = useState(false);
  const critical = snapshot && snapshot.disk.available_bytes < 10 * 1024 * 1024 * 1024;
  const visibleHotspots = storage?.hotspots.slice(0, showAll ? undefined : 8) ?? [];
  return <>
    {critical && <div className="storage-warning" role="alert">
      <AlertTriangle aria-hidden="true" />
      <div><strong>Storage is critically low</strong><span>{formatBytes(snapshot.disk.available_bytes)} remains. Analyze first—large developer data is not automatically safe to delete.</span></div>
    </div>}
    <section className="storage-card" aria-labelledby="storage-title">
      <div className="section-heading">
        <div><p className="eyebrow">Read-only diagnosis</p><h2 id="storage-title">What is using your disk</h2></div>
        <div className="section-actions">
          <button className="secondary-button" type="button" onClick={() => void onAnalyze()} disabled={analyzing || scanning}>
            {analyzing ? <LoaderCircle className="spin" aria-hidden="true" /> : <HardDrive aria-hidden="true" />}
            {analyzing ? "Analyzing…" : storage ? "Analyze again" : "Analyze storage"}
          </button>
          <button className="primary-button" type="button" onClick={() => void onReviewCleanup()} disabled={analyzing || scanning}>
            {scanning ? <LoaderCircle className="spin" aria-hidden="true" /> : <Search aria-hidden="true" />}
            {analyzing ? "Finding…" : "Find cleanup options"}
          </button>
        </div>
      </div>
      {!storage && !analyzing && <div className="storage-intro">
        <h3>Find the large known locations first</h3>
        <p>Mac Cleaner measures allocated size from metadata and reads only small signatures in explicitly recognized developer caches. It reports rebuildable output, tool-managed data, and protected development state separately.</p>
      </div>}
      {analyzing && <div className="analysis-progress" role="status"><LoaderCircle className="spin" aria-hidden="true" /><div><strong>Measuring known hotspots…</strong><span>Large project dependency trees can take a minute. Nothing is moved or deleted.</span></div></div>}
      {storage && !analyzing && <>
        <div className="storage-summary">
          <div><span>Available now</span><strong>{formatBytes(snapshot?.disk.available_bytes ?? storage.available_bytes)}</strong></div>
          <div><span>Needs individual review</span><strong>{formatBytes(storage.reviewable_bytes)}</strong></div>
          <p>“Reviewable” is not a promise of safe recovery. Confirm the project is inactive and use the recommended owning tool.</p>
        </div>
        <div className="hotspot-list">{visibleHotspots.map((hotspot) => <StorageHotspotCard key={hotspot.id} hotspot={hotspot} onOpenOwner={onOpenOwner} onPruneUv={onPruneUv} onCleanSimulatorRecordings={onCleanSimulatorRecordings} toolOperation={toolOperation} uvStatus={uvStatus} simulatorStatus={simulatorStatus} />)}</div>
        {storage.hotspots.length > 8 && <button className="secondary-button hotspot-toggle" type="button" onClick={() => setShowAll((current) => !current)}>
          {showAll ? "Show the largest 8" : `Show all ${storage.hotspots.length} locations`}
        </button>}
        {storage.warnings.length > 0 && <details className="skipped"><summary>{storage.warnings.length} location(s) could not be fully measured</summary>{storage.warnings.map((warning) => <p key={warning}>{warning}</p>)}</details>}
      </>}
    </section>
  </>;
}

function StorageHotspotCard({ hotspot, onOpenOwner, onPruneUv, onCleanSimulatorRecordings, toolOperation, uvStatus, simulatorStatus }: { hotspot: StorageAnalysis["hotspots"][number]; onOpenOwner: (owner: StorageOwner) => Promise<void>; onPruneUv: () => Promise<void>; onCleanSimulatorRecordings: () => Promise<void>; toolOperation: ToolOperation; uvStatus: string | null; simulatorStatus: string | null }) {
  const labels: Record<StorageDisposition, string> = { reviewable: "Reviewable", "tool-managed": "Use owning tool", protected: "Protected" };
  const Icon = hotspot.disposition === "reviewable" ? Wrench : hotspot.disposition === "tool-managed" ? Database : LockKeyhole;
  const owner = hotspot.display_name === "Docker Desktop virtual disk" ? "docker" : hotspot.display_name === "Apple simulator devices" ? "xcode" : null;
  return <article className={`hotspot ${hotspot.disposition}`}>
    <div className="hotspot-top"><span className="hotspot-icon"><Icon aria-hidden="true" /></span><div><strong>{hotspot.display_name}</strong><code>{hotspot.path}</code></div><b>{formatBytes(hotspot.bytes)}</b></div>
    <div className="hotspot-detail"><span className={`storage-badge ${hotspot.disposition}`}>{labels[hotspot.disposition]}</span><p>{hotspot.reason}</p></div>
    <div className="recommendation"><strong>Suggested next step</strong><span>{hotspot.recommendation}</span><small>Cost: {hotspot.recovery_cost}</small></div>
    {hotspot.display_name === "uv package cache" && <>
      <button className="primary-button hotspot-action" type="button" onClick={() => void onPruneUv()} disabled={toolOperation !== null}>
        {toolOperation === "uv" ? <LoaderCircle className="spin" aria-hidden="true" /> : <Wrench aria-hidden="true" />}
        {toolOperation === "uv" ? "Pruning uv…" : "Prune unused uv cache"}
      </button>
      {uvStatus && <p className={`tool-action-status ${statusTone(uvStatus)}`} role="status" aria-live="polite">{uvStatus}</p>}
    </>}
    {hotspot.display_name === "Simulator test recordings" && <>
      <button className="danger-button hotspot-action" type="button" onClick={() => void onCleanSimulatorRecordings()} disabled={toolOperation !== null}>
        {toolOperation === "simulator" ? <LoaderCircle className="spin" aria-hidden="true" /> : <Wrench aria-hidden="true" />}
        {toolOperation === "simulator" ? "Preparing recording cleanup…" : "Review and remove old recordings"}
      </button>
      {simulatorStatus && <p className={`tool-action-status ${statusTone(simulatorStatus)}`} role="status" aria-live="polite">{simulatorStatus}</p>}
    </>}
    {owner && <button className="secondary-button hotspot-action" type="button" onClick={() => void onOpenOwner(owner)}><ExternalLink aria-hidden="true" /> {owner === "docker" ? "Open Docker Desktop" : "Open Xcode"}</button>}
  </article>;
}

function statusTone(status: string) {
  if (status.startsWith("Completed") || status.startsWith("Last completed") || status.startsWith("Last cleanup")) return "success";
  if (status.startsWith("Cleanup failed")) return "error";
  if (status.startsWith("Cancelled")) return "neutral";
  return "info";
}
