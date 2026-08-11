import { ArchiveRestore, Trash2 } from "lucide-react";
import { formatBytes, formatDate } from "../format";
import type { Phase } from "../hooks/useMacCleaner";
import type { QuarantineAction, QuarantineEntry } from "../types";

interface Props {
  entries: QuarantineEntry[];
  phase: Phase;
  onAction: (entryId: string, action: QuarantineAction) => Promise<void>;
}

export function QuarantineSection({ entries, phase, onAction }: Props) {
  return <section className="history" aria-labelledby="quarantine-title">
    <div className="section-heading">
      <div><p className="eyebrow">Recovery boundary</p><h2 id="quarantine-title">Quarantine</h2></div>
      <ArchiveRestore aria-hidden="true" />
    </div>
    <p className="muted">Moving an item here does not free disk space. Restore it, or permanently delete one exact item after another native confirmation.</p>
    {entries.length === 0 ? <p className="muted">Quarantine is empty.</p> : entries.map((entry) => (
      <article className="quarantine-row" key={entry.id}>
        <div>
          <strong>{formatBytes(entry.bytes)} · {entry.files} entries</strong>
          <code>{entry.original_path}</code>
          <small>Quarantined {formatDate(entry.quarantined_at)}</small>
        </div>
        <div className="quarantine-actions">
          <button className="secondary-button" type="button" disabled={phase !== "idle"} onClick={() => void onAction(entry.id, "restore")}>
            <ArchiveRestore aria-hidden="true" /> Restore
          </button>
          <button className="danger-button" type="button" disabled={phase !== "idle"} onClick={() => void onAction(entry.id, "purge")}>
            <Trash2 aria-hidden="true" /> Delete permanently
          </button>
        </div>
      </article>
    ))}
  </section>;
}
