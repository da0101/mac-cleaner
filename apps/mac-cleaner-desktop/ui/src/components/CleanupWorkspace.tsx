import { CheckCircle2, ChevronRight, Database, LoaderCircle, RefreshCw, ShieldCheck } from "lucide-react";
import { formatBytes } from "../format";
import type { Phase } from "../hooks/useMacCleaner";
import type { ScanReport } from "../types";

interface Props {
  scan: ScanReport | null;
  selected: Set<string>;
  selectedBytes: number;
  phase: Phase;
  onScan: () => Promise<void>;
  onToggleCandidate: (id: string) => void;
  onReviewPlan: () => Promise<void>;
}

export function CleanupWorkspace({ scan, selected, selectedBytes, phase, onScan, onToggleCandidate, onReviewPlan }: Props) {
  return <section className="workspace" aria-labelledby="cleanup-title">
    <div className="section-heading">
      <div><p className="eyebrow">Direct cleanup</p><h2 id="cleanup-title">Files eligible for quarantine</h2></div>
      <button className="secondary-button" type="button" onClick={() => void onScan()} disabled={phase !== "idle"}>
        {phase === "scanning" ? <LoaderCircle className="spin" aria-hidden="true" /> : <RefreshCw aria-hidden="true" />}
        {phase === "scanning" ? "Analyzing…" : "Refresh all cleanup options"}
      </button>
    </div>

    {!scan && <EmptyState onScan={onScan} />}
    {scan && scan.candidates.length === 0 && (
      <div className="empty-state">
        <CheckCircle2 aria-hidden="true" />
        <h3>No direct quarantine candidates</h3>
        <p>Use the actionable storage suggestions above for tool-managed caches and recordings. The strict file catalog found nothing else eligible to move safely.</p>
      </div>
    )}
    {scan && scan.candidates.length > 0 && (
      <div className="candidate-list">
        {scan.candidates.map((candidate) => (
          <label className={`candidate ${selected.has(candidate.id) ? "selected" : ""}`} key={candidate.id}>
            <input type="checkbox" checked={selected.has(candidate.id)} onChange={() => onToggleCandidate(candidate.id)} />
            <span className="candidate-check" aria-hidden="true"><CheckCircle2 /></span>
            <span className="candidate-content">
              <span className="candidate-row"><strong>{candidate.display_name}</strong><span className="review-badge">Review required</span></span>
              <code>{candidate.lexical_path}</code>
              <span className="candidate-meta"><span>{formatBytes(candidate.bytes)}</span><span>{candidate.files} entries</span><span>{candidate.owner}</span></span>
              <span className="recovery"><Database aria-hidden="true" /> {candidate.recovery}</span>
            </span>
          </label>
        ))}
      </div>
    )}

    {scan && scan.skipped.length > 0 && (
      <details className="skipped">
        <summary>{scan.skipped.length} protected or unsafe entries skipped</summary>
        {scan.skipped.map((item, index) => <p key={`${item.path}-${index}`}><code>{item.path}</code><br />{item.reason}</p>)}
      </details>
    )}

    {selected.size > 0 && (
      <div className="action-bar">
        <div><strong>{selected.size} selected</strong><span>{formatBytes(selectedBytes)} moves to recoverable quarantine</span></div>
        <button className="primary-button" type="button" onClick={() => void onReviewPlan()} disabled={phase !== "idle"}>
          Review exact plan <ChevronRight aria-hidden="true" />
        </button>
      </div>
    )}
  </section>;
}

function EmptyState({ onScan }: { onScan: () => Promise<void> }) {
  return <div className="empty-state">
    <ShieldCheck aria-hidden="true" />
    <h3>Analyze all cleanup options</h3>
    <p>Mac Cleaner will inspect known storage hotspots and exact approved file locations. Analysis never moves or deletes files.</p>
    <button className="primary-button" type="button" onClick={() => void onScan()}>
      <RefreshCw aria-hidden="true" /> Analyze cleanup options
    </button>
  </div>;
}
