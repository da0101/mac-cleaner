---
domain_id: dom-local-dashboard
slug: local-dashboard
status: retired
repo_ids: [mac-cleaner]
related_domain_slugs: [storage-cleanup, memory-optimization]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# local-dashboard

<!-- updated: 2026-11-16 — retired and replaced by the native macOS shell. -->

Historical localhost dashboard for scan results, RAM/process summaries, history, and manual actions. The current app uses SwiftUI/AppKit and opens no HTTP listener or web view.

## Replacement key files

- `apps/mac-cleaner-macos/Sources/RootView.swift`
- `apps/mac-cleaner-macos/Sources/MenuBarView.swift`
- `apps/mac-cleaner-macos/Sources/AppModel.swift`
- `apps/mac-cleaner-engine/src/main.rs`

## Locked contracts

- Open no network listener.
- Public release defaults to read-only analysis; cleanup remains explicitly reviewed.
- Do not accept arbitrary deletion paths from any client.
- Memory display should keep free RAM and available/reusable RAM visible as distinct values.
- Menu bar cleanup actions must keep the same confirmation-first behavior as dashboard cleanup controls.
- Automatic RAM purge and storage cleanup schedules are retired.
