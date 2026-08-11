import { CheckCircle2, History } from "lucide-react";
import { formatBytes, formatDate } from "../format";
import type { ExecutionReceipt, ToolCleanupReceipt } from "../types";

interface Props {
  history: ExecutionReceipt[];
  toolHistory: ToolCleanupReceipt[];
}

export function ActivitySection({ history, toolHistory }: Props) {
  const activity = [
    ...history.map((receipt) => ({
      id: receipt.plan_id,
      completedAt: receipt.completed_at,
      title: `${receipt.items.length} item(s) moved to quarantine`,
      detail: "Recoverable cleanup",
    })),
    ...toolHistory.map((receipt) => ({
      id: receipt.plan_id,
      completedAt: receipt.completed_at,
      title: receipt.summary,
      detail: `${formatBytes(receipt.reclaimed_bytes)} reclaimed · ${receipt.target_path}`,
    })),
  ].sort((left, right) => Date.parse(right.completedAt) - Date.parse(left.completedAt));

  return <section className="history" aria-labelledby="history-title">
    <div className="section-heading">
      <div><p className="eyebrow">Audit trail</p><h2 id="history-title">Recent activity</h2></div>
      <History aria-hidden="true" />
    </div>
    {activity.length === 0 ? <p className="muted">No cleanup operations have completed yet.</p> : activity.slice(0, 8).map((item) => (
      <div className="history-row" key={item.id}>
        <CheckCircle2 aria-hidden="true" />
        <span><strong>{item.title}</strong><small>{item.detail}</small></span>
        <span className="activity-meta"><b>Completed</b><time dateTime={item.completedAt}>{formatDate(item.completedAt)}</time></span>
      </div>
    ))}
  </section>;
}
