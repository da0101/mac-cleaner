import { MonitorCog } from "lucide-react";
import { formatBytes, processMemorySeverity } from "../format";
import type { SystemSnapshot } from "../types";

export function ProcessMonitor({ snapshot }: { snapshot: SystemSnapshot | null }) {
  return <section className="process-card" aria-labelledby="process-title">
    <div className="section-heading">
      <div><p className="eyebrow">Read-only monitor</p><h2 id="process-title">Top memory processes</h2></div>
      <MonitorCog aria-hidden="true" />
    </div>
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
    <p className="monitor-note">Mac Cleaner cannot signal, quit, or kill these processes.</p>
  </section>;
}
