---
domain_id: dom-memory-optimization
slug: memory-optimization
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [native-macos-shell, rust-application]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# memory-optimization

Reports conservative available/reclaimable/compressed/swap memory, derives pressure, and ranks heavy processes while protecting active developer work. Recovery is advisory: the app may notify and open Activity Monitor but never purges memory or terminates applications.

## Key files

- `crates/cleaner-system-macos/src/lib.rs` captures memory, disk, and protected-process snapshots.
- `crates/cleaner-domain/src/system.rs` defines portable snapshot and pressure contracts.
- `apps/mac-cleaner-engine/src/main.rs` exposes read-only status through protocol v1.
- `apps/mac-cleaner-macos/Sources/AppModel.swift` owns 30-second refresh and pressure notifications.
- `apps/mac-cleaner-macos/Sources/OverviewView.swift` and `MenuBarView.swift` render native metrics and Activity Monitor guidance.

## Locked contracts

- Use conservative available/reclaimable RAM and swap for health, not raw free pages alone.
- Keep free, available, compressed, reclaimable, and swap meanings distinct.
- Menu bar process visibility is read-only; do not add process kill/quit actions there without explicit confirmation design.
- Do not auto-kill developer tools or shell jobs.
- Editors, terminals, agents, compilers, and active development jobs are protected.
- Never invoke `purge`, generate artificial pressure, send process signals, or claim that disk-cache flushing frees application RAM.
