import { Gauge, HardDrive, MemoryStick, RefreshCw, ShieldCheck } from "lucide-react";
import type { ReactNode } from "react";
import { availabilitySeverity, formatBytes, formatPercent } from "../format";
import type { MemorySeverity } from "../format";
import type { SystemSnapshot } from "../types";

export function SystemOverview({ snapshot }: { snapshot: SystemSnapshot | null }) {
  const pressure = snapshot?.memory.pressure ?? "normal";
  const pressureLabel = pressure === "normal" ? "Normal" : pressure === "warning" ? "Elevated" : "Critical";
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
        severity={pressure === "normal" ? "normal" : pressure === "warning" ? "high" : "very-high"}
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

    <div className={`memory-policy ${pressure}`} role="status" aria-live="polite">
      <RefreshCw aria-hidden="true" />
      <div>
        <div className="memory-policy-heading"><strong>Automatic memory recovery is active</strong><b>{pressureLabel} pressure</b></div>
        <span>Mac Cleaner checks every 30 seconds while macOS recycles inactive and compressed memory. It alerts on sustained pressure and never generates pressure or closes apps automatically.</span>
        {snapshot && <div className="memory-recovery-stats" aria-label="Automatic memory recovery details">
          <span><Gauge aria-hidden="true" /><b>{formatBytes(snapshot.memory.reclaimable_bytes)}</b> reclaimable</span>
          <span><RefreshCw aria-hidden="true" /><b>{formatBytes(snapshot.memory.compressed_bytes)}</b> compressed</span>
          <span><HardDrive aria-hidden="true" /><b>{formatBytes(snapshot.memory.swap_used_bytes)}</b> swap</span>
        </div>}
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
