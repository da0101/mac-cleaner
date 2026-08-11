import { HardDrive, MemoryStick, ShieldCheck } from "lucide-react";
import type { ReactNode } from "react";
import { availabilitySeverity, formatBytes, formatPercent } from "../format";
import type { MemorySeverity } from "../format";
import type { SystemSnapshot } from "../types";

export function SystemOverview({ snapshot }: { snapshot: SystemSnapshot | null }) {
  return <>
    <section className="hero" aria-labelledby="overview-title">
      <div>
        <p className="eyebrow">System overview</p>
        <h1 id="overview-title">See what is safe before cleaning.</h1>
        <p className="lede">Mac Cleaner scans a small, versioned catalog. Unknown paths and development work stay protected.</p>
      </div>
      <div className="safety-pill"><ShieldCheck aria-hidden="true" /> Automatic deletion off</div>
    </section>

    <section className="metric-grid" aria-label="System health">
      <MetricCard
        icon={<MemoryStick aria-hidden="true" />}
        label="Available memory"
        value={snapshot ? formatBytes(snapshot.memory.available_bytes) : "Loading…"}
        percent={snapshot ? formatPercent(snapshot.memory.used_bytes, snapshot.memory.total_bytes) : 0}
        detail={snapshot ? `${formatBytes(snapshot.memory.free_bytes)} completely free · ${formatBytes(snapshot.memory.total_bytes)} installed` : "Read-only metric"}
        severity={snapshot ? availabilitySeverity(snapshot.memory.available_bytes, snapshot.memory.total_bytes) : "normal"}
      />
      <MetricCard
        icon={<HardDrive aria-hidden="true" />}
        label="Available disk"
        value={snapshot ? formatBytes(snapshot.disk.available_bytes) : "Loading…"}
        percent={snapshot ? formatPercent(snapshot.disk.total_bytes - snapshot.disk.available_bytes, snapshot.disk.total_bytes) : 0}
        detail={snapshot ? `${formatBytes(snapshot.disk.total_bytes)} total` : "Read-only metric"}
        severity={snapshot ? availabilitySeverity(snapshot.disk.available_bytes, snapshot.disk.total_bytes) : "normal"}
      />
    </section>

    <div className="memory-policy" role="status">
      <ShieldCheck aria-hidden="true" />
      <div>
        <strong>Automatic memory watchdog is on</strong>
        <span>Checks every 30 seconds and alerts below 1 GB available. macOS manages cached RAM; Mac Cleaner never purges memory or closes apps automatically.</span>
      </div>
    </div>
  </>;
}

interface MetricCardProps {
  icon: ReactNode;
  label: string;
  value: string;
  percent: number;
  detail: string;
  severity: MemorySeverity;
}

function MetricCard({ icon, label, value, percent, detail, severity }: MetricCardProps) {
  const status = severity === "normal" ? "Healthy" : severity === "high" ? "Low" : "Critical";
  return <article className={`metric-card ${severity}`}>
    <span className="metric-icon">{icon}</span>
    <div className="metric-copy">
      <div className="metric-heading"><span className="metric-label">{label}</span><span className="metric-status">{status}</span></div>
      <strong>{value}</strong>
      <small>{detail}</small>
      <div className="progress" role="progressbar" aria-label={`${label} used`} aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent}>
        <span style={{ width: `${percent}%` }} />
      </div>
    </div>
  </article>;
}
