import { ExternalLink, MonitorCog, ShieldAlert } from "lucide-react";
import { formatBytes, processMemorySeverity } from "../format";
import type { SystemSnapshot } from "../types";

interface Props {
  snapshot: SystemSnapshot | null;
  onOpenActivityMonitor: () => Promise<void>;
}

export function ProcessMonitor({ snapshot, onOpenActivityMonitor }: Props) {
  const pressure = snapshot?.memory.pressure ?? "normal";
  const suggestions = snapshot?.processes.filter((process) => !process.protected).slice(0, 3) ?? [];
  return <section className="process-card" aria-labelledby="process-title">
    <div className="section-heading">
      <div><p className="eyebrow">Read-only monitor</p><h2 id="process-title">Top memory processes</h2></div>
      <button className="secondary-button" type="button" onClick={() => void onOpenActivityMonitor()}><ExternalLink aria-hidden="true" /> Open Activity Monitor</button>
    </div>
    {snapshot && <div className={`recovery-guidance ${pressure}`}>
      {pressure === "normal" ? <MonitorCog aria-hidden="true" /> : <ShieldAlert aria-hidden="true" />}
      <div>
        <strong>{pressure === "normal" ? "No intervention recommended" : pressure === "warning" ? "Memory pressure is elevated" : "Memory pressure is critical"}</strong>
        <span>{pressure === "normal"
          ? "macOS is recycling memory normally. Cached and compressed RAM remains available when applications need it."
          : suggestions.length > 0
            ? `Review ${suggestions.map((process) => process.name).join(", ")} first. Mac Cleaner keeps developer jobs protected and leaves the final decision to you.`
            : "Review the largest applications in Activity Monitor. Protected developer jobs will not be closed automatically."}</span>
      </div>
    </div>}
    {!snapshot ? <p className="muted">Loading process snapshot…</p> : (
      <div className="process-list" role="table" aria-label="Top processes by resident memory">
        <div className="process-row process-header" role="row">
          <span role="columnheader">Process</span>
          <span role="columnheader">Memory</span>
          <span role="columnheader">CPU</span>
          <span role="columnheader">Protection</span>
        </div>
        {snapshot.processes.map((process) => {
          const severity = processMemorySeverity(process.memory_bytes, snapshot.memory.total_bytes);
          return <div className="process-row" role="row" key={`${process.pid}-${process.start_time_unix}`}>
            <span className="process-name" role="cell"><strong>{process.name}</strong><small>PID {process.pid}</small></span>
            <span className={`process-memory ${severity}`} role="cell">
              {formatBytes(process.memory_bytes)}
              <small>{severity === "normal" ? "Normal" : severity === "very-high" ? "Very high" : "High"}</small>
            </span>
            <span role="cell">{process.cpu_percent.toFixed(1)}% CPU</span>
            <span className={process.protected ? "process-status protected" : "process-status"} role="cell">
              {process.protected ? "Developer protected" : "Read only"}
            </span>
          </div>;
        })}
      </div>
    )}
    <p className="monitor-note">Recovery suggestions are advisory. Quitting an app remains a user-confirmed macOS action.</p>
  </section>;
}
