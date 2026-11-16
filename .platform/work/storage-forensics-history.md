---
stream_id: stream-storage-forensics-history
slug: storage-forensics-history
type: feature
status: closure-review
agent_owner: codex
domain_slugs: [storage-forensics, rust-application, safe-cleanup-policy]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: feature/full-rust-rewrite
created_at: 2026-11-16
updated_at: 2026-11-16
closure_approved: false
---

# storage-forensics-history

## Scope

- Add read-only, path-level storage findings for recurring developer-machine growth sources.
- Persist bounded local snapshots and show size deltas since the previous/baseline scan.
- Add a compact, filterable desktop list with exact paths, risk, ownership, growth, and copy/export output.
- Preserve the existing cleanup authorization boundary; this feature does not add arbitrary deletion.
- Keep repositories, GreyBox Docker data, mobile SDKs/runtimes, and unknown user data protected.

## Done criteria

- [x] Repeated scans attribute measured growth to stable finding identifiers and exact paths.
- [x] UI supports compact sorting/filtering, path reveal, and one-click copy of a review report.
- [x] Snapshot retention is bounded and coverage changes are distinguished from real growth.
- [x] Rust/unit/UI tests cover main, empty, error, and delta cases without deleting real files.
- [x] Live macOS QA explains current storage growth without exposing secrets or granting cleanup authority.
- [x] `.platform/memory/log.md` appended
- [x] `decisions.md` updated if any architectural choices were made

## Key decisions

2026-11-16 — Keep storage forensics read-only and separate from cleanup authorization — size and growth alone never prove that a path is safe to delete.

2026-11-16 — Compare a hotspot only when the same stable target was measured in both snapshots — newly tracked paths are labeled as coverage changes instead of growth.

2026-11-16 — Show whole-volume used-space change separately from summed hotspot changes — any remainder is an explicit accounting gap, not invented attribution.

## Resume state

- **Last updated:** 2026-11-16 by codex
- **What just happened:** Completed stable storage snapshots, baseline/previous comparisons, bounded retention, accounting gaps, manifest-aware generated-output findings, protocol exposure, and the native filterable/copyable SwiftUI table. Rust and Swift tests plus live read-only analysis pass.
- **Current focus:** Owner closure review alongside the native shell migration.
- **Next action:** Owner may approve commit and close this stream after reviewing the native storage screen.
- **Blockers:** None.

## Progress log

2026-11-16 — Registered the feature stream and storage-forensics domain; no product code changed.

2026-11-16 — Audited the Rust domain, filesystem scanner, service, SQLite store, Tauri commands, React storage UI, and existing tests.

2026-11-16 — Current implementation is ephemeral: scans are not persisted, current path-based IDs have no comparable coverage metadata, and the card UI has no history, filtering, baseline, or copyable evidence report.

2026-11-16 — The live storage audit found repeat growth in Docker test assets, TakeCare worktree dependencies, Yarn cache, and developer support data. It also reproduced a mount-traversal false positive under `/Library`, reinforcing explicit scan boundaries.

2026-11-16 — Completed targeted research on APFS clones, hidden/shared space, and allocated-size reporting.

2026-11-16 — Owner approved the research-backed implementation plan; implementation started without cleanup or deletion authority.

2026-11-16 — Owner replaced the Tauri/React presentation requirement with pure SwiftUI. Rust snapshot, catalog, comparison, and SQLite work remains in scope; React presentation work will be discarded during the approved shell migration.

2026-11-16 — Finished Rust snapshot IDs, comparison accounting, SQLite retention/baselines, expanded developer-storage catalog, native SwiftUI filtering/copy/baseline controls, protocol tests, and live read-only QA. No cleanup authority was added.

## Open questions

- None.

---

## 🔍 Audit Report

_Status: implementation and local audit complete; awaiting owner closure approval._
