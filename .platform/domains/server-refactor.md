---
domain_id: dom-server-refactor
slug: server-refactor
status: retired
repo_ids: [mac-cleaner]
related_domain_slugs: [local-dashboard, memory-optimization, storage-cleanup, background-ai-optimizer, browser-tab-optimizer]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# server-refactor

<!-- updated: 2026-11-16 — historical v1 modularization; the Python server was removed. -->

## What this domain does

Historically defined how the monolithic Python dashboard server was split into modules. That server and its assets are retired; the maintainability lesson survives in the Rust crate and native-view boundaries.

## Replacement boundaries

- `apps/mac-cleaner-macos/` owns native presentation and lifecycle.
- `apps/mac-cleaner-engine/` owns the constrained client boundary.
- `crates/cleaner-service/` owns orchestration; domain, filesystem, policy, store, and system concerns remain separate crates.
- Cleanup stays confirmation-first and accepts no client-supplied paths.

## Historical target boundaries

The following constraints document the removed server and are retained only to explain the earlier modularization:

- Configuration/state/persistence should be isolated from HTTP routing.
- Storage cleanup wrappers should sit near scanner integration.
- Memory/process logic should not live beside dashboard HTML.
- AI and Chrome routes should call existing `ai_advisor.py` and `browser_tabs.py` helpers.
- Dashboard HTML/CSS/JS lives under `mac_cleaner_server/assets/` and is served by the local dashboard router.

## Historical note

- The localhost API contract and Python module structure no longer constrain active development.
- Cleanup safety lessons remain locked in `safe-cleanup-policy`.
