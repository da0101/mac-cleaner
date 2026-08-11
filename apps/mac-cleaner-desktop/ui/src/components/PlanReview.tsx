import { LoaderCircle, Play } from "lucide-react";
import { formatBytes, formatDate } from "../format";
import type { Phase } from "../hooks/useMacCleaner";
import type { CleanupPlan } from "../types";

interface Props {
  plan: CleanupPlan;
  phase: Phase;
  onExecute: () => Promise<void>;
}

export function PlanReview({ plan, phase, onExecute }: Props) {
  return <section className="plan-card" aria-labelledby="plan-title">
    <div><p className="eyebrow">Final review</p><h2 id="plan-title">Plan {plan.id}</h2></div>
    <p>{formatBytes(plan.total_bytes)} across {plan.total_files} entries. Expires {formatDate(plan.expires_at)}.</p>
    <ul>{plan.items.map((item) => <li key={item.id}><code>{item.lexical_path}</code><span>{item.recovery}</span></li>)}</ul>
    <button className="primary-button warning" type="button" onClick={() => void onExecute()} disabled={phase !== "idle"}>
      {phase === "executing" ? <LoaderCircle className="spin" aria-hidden="true" /> : <Play aria-hidden="true" />}
      {phase === "executing" ? "Revalidating…" : "Open native confirmation"}
    </button>
  </section>;
}
