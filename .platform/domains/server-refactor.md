---
domain_id: dom-server-refactor
slug: server-refactor
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [local-dashboard, memory-optimization, storage-cleanup, background-ai-optimizer, browser-tab-optimizer]
created_at: 2026-04-29
updated_at: 2026-04-29
---

# server-refactor

## What this domain does

Defines how the monolithic dashboard server is split into maintainable modules without changing local behavior.

## Cross-layer touch points

- `server.py` now stays as the executable entrypoint; `mac_cleaner_server/` owns CLI parsing, global state, settings/history persistence, storage cleanup APIs, memory/process analysis, AI refresh, Chrome tab APIs, dashboard assets, HTTP routing, background loops, and process startup.
- `start` launches `server.py` directly and must keep working.
- `tests/` import service modules directly when possible.
- Dashboard routes must continue binding to `127.0.0.1:3333`.
- Cleanup actions must remain confirmation/preview-oriented and must not accept arbitrary paths.

## Target boundaries

- Configuration/state/persistence should be isolated from HTTP routing.
- Storage cleanup wrappers should sit near scanner integration.
- Memory/process logic should not live beside dashboard HTML.
- AI and Chrome routes should call existing `ai_advisor.py` and `browser_tabs.py` helpers.
- Dashboard HTML/CSS/JS lives under `mac_cleaner_server/assets/` and is served by the local dashboard router.

## Non-goals

- Do not rewrite the server framework.
- Do not change the localhost API contract unless tests and README are updated.
- Do not alter cleanup safety tiers or automatic action policy.
