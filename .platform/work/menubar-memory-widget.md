---
stream_id: stream-menubar-memory-widget
slug: menubar-memory-widget
type: bug
status: in-progress
agent_owner: codex
domain_slugs: [memory-optimization, local-dashboard]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: fix/menubar-memory-widget
created_at: 2026-05-04
updated_at: 2026-05-04
closure_approved: false
---

# menubar-memory-widget

## Scope
- Restore a clean terminal startup banner for the dashboard command.
- Make dashboard memory display match the menu bar widget's free/available RAM model.
- Keep the menu bar RAM icon vertical and readable in the macOS status bar.
- Add menu bar stats/actions parity for the dashboard's common memory and cleanup controls.
- Out of scope: killing/closing processes from the menu bar and AI recommendation policy.

## Done criteria
- [x] Startup banner brands the command as Mac Cleaner without the internal "SERVER" label.
- [x] Dashboard memory card shows raw free RAM and available RAM together.
- [x] Dashboard free RAM color thresholds match the menu bar thresholds.
- [x] Menu bar RAM icon is drawn as a vertical module.
- [x] Menu bar dropdown shows garbage accumulation, RAM purge timer, and top 5 memory consumers.
- [x] Menu bar exposes confirmed Purge RAM, Clean Garbage, Docker Prune, Refresh Stats, and Open Dashboard actions.
- [x] Dashboard and menu bar countdowns read from the same server schedule through a lightweight API.
- [x] Menu bar can explicitly enable/pause auto-clean without stale toggle inversions.
- [x] `python3 -m unittest` passes.
- [x] `python3 -m py_compile scanner.py cleaner.py server.py ai_cleaner.py ai_advisor.py browser_tabs.py mac_cleaner_server/*.py menubar.py` passes.
- [x] `.platform/memory/log.md` appended.
- [x] `decisions.md` update not needed; no architectural choices were made.

## Key decisions
- 2026-05-04 - Reuse the existing menu bar stream - ACTIVE.md already had the stream row, so this file restores the missing tracked context instead of creating a duplicate.

## Resume state
- **Last updated:** 2026-05-04 by codex
- **What just happened:** Replaced stale-prone auto-clean toggles with explicit set/status APIs, made menu bar sync sticky across transient localhost failures, and kept action rows high contrast; tests pass.
- **Current focus:** Ready for user review.
- **Next action:** Restart the dashboard server and menu bar widget, then review the expanded dropdown.
- **Blockers:** none

## Progress log
- 2026-05-04 10:30 - Restored missing stream file and scoped requested display bugs.
- 2026-05-04 10:45 - Implemented display fixes and verified unit/compile checks.
- 2026-05-04 10:55 - Matched agentboard banner style and reduced menu bar icon width after visual review.
- 2026-05-04 11:00 - Switched to a distinct compact startup badge and scaled the menu bar glyph down again.
- 2026-05-04 11:05 - Restored a fancy banner without copying agentboard's pixel-art style.
- 2026-05-04 12:45 - Added menu bar stats/actions parity with dashboard controls.
- 2026-05-04 13:05 - Moved auto-clean countdowns into shared server state and added menu bar auto-clean toggle.
- 2026-05-04 13:35 - Added explicit auto-clean status/set API and sticky menu bar sync to fix stale paused state.

## Open questions
_Things blocked on user input. Remove when resolved._
