---
stream_id: stream-server-refactor
slug: server-refactor
type: refactor
status: closure-review
agent_owner: codex
domain_slugs: [server-refactor, local-dashboard, memory-optimization, storage-cleanup, background-ai-optimizer, browser-tab-optimizer]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: main
created_at: 2026-04-29
updated_at: 2026-04-29
closure_approved: false
---

# server-refactor

## Scope

- Split `server.py` into smaller modules with clear ownership boundaries.
- Preserve CLI behavior, local dashboard routes, safety gates, and tests.
- Keep `server.py` as the thin executable entrypoint so `./start` and `mac-cleaner` keep working.
- Update tests/imports and docs where module paths change.
- Out of scope: replacing `http.server` with another framework or changing cleanup policy.

## Done criteria

- [x] `server.py` is reduced to a small entrypoint/router surface and no file introduced by this refactor is a new large monolith.
- [x] Existing dashboard/API behavior is preserved.
- [x] `python3 -m unittest` passes.
- [x] Python compile check passes for changed modules.
- [x] Dashboard smoke check passes on localhost.
- [x] `.platform/memory/log.md` appended.
- [x] `decisions.md` updated if any architectural choices were made.

## Key decisions

_Append-only. Format: `YYYY-MM-DD — <decision> — <rationale>`_

2026-04-29 — Split server code into `mac_cleaner_server/` package — keeps `server.py` as an executable shim and separates state, routing, services, runtime, and dashboard assets.

## Resume state
_Overwritten by `ab checkpoint` — the compact payload the next agent reads first. Keep this block under ~10 lines._

- **Last updated:** 2026-04-29 by danilulmashev
- **What just happened:** Prepared v1.1.0 release docs and updated the terminal server banner to v1.1.
- **Current focus:** —
- **Next action:** Create and push the v1.1.0 release tag on the release-docs commit.
- **Blockers:** none

## Progress log

2026-04-29 19:53 — Prepared v1.1.0 release docs and updated the terminal server banner to v1.1.

2026-04-29 19:53 — (auto) 83f41b0: Prepare v1.1.0 release docs

2026-04-29 19:46 — Moved the dashboard primary action bar above the AI and Chrome optimizer panels and committed f345edd.

2026-04-29 19:34 — (auto) 8df1848: Add AI optimizers and split dashboard server

2026-04-29 19:13 — Registered server-refactor stream/domain and completed initial local plus Python docs research for splitting the 2k-line server.py.

2026-04-29 19:25 — Split `server.py` into focused `mac_cleaner_server/` modules and static dashboard assets; tests and compile checks pass.

2026-04-29 19:35 — Verified refactored dashboard with HTTP and Playwright smoke; all done criteria are checked.

2026-04-29 19:00 — Registered server refactor stream and domain.

## Open questions

- Resolved: dashboard HTML/CSS/JS now lives in static assets under `mac_cleaner_server/assets/`.

---

## 🔍 Audit Report

_Status: not yet run_
