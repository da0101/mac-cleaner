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
- Keep the menu bar RAM icon horizontal and readable in the macOS status bar.
- Out of scope: cleanup behavior, kill/delete actions, and AI recommendation policy.

## Done criteria
- [x] Startup banner brands the command as Mac Cleaner without the internal "SERVER" label.
- [x] Dashboard memory card shows raw free RAM and available RAM together.
- [x] Dashboard free RAM color thresholds match the menu bar thresholds.
- [x] Menu bar RAM icon is drawn as a horizontal module.
- [x] `python3 -m unittest` passes.
- [x] `python3 -m py_compile scanner.py cleaner.py server.py ai_cleaner.py ai_advisor.py browser_tabs.py mac_cleaner_server/*.py menubar.py` passes.
- [x] `.platform/memory/log.md` appended.
- [x] `decisions.md` update not needed; no architectural choices were made.

## Key decisions
- 2026-05-04 - Reuse the existing menu bar stream - ACTIVE.md already had the stream row, so this file restores the missing tracked context instead of creating a duplicate.

## Resume state
- **Last updated:** 2026-05-04 by codex
- **What just happened:** Restored a fancy slanted Mac Cleaner splash with distinct green/blue colors, kept the menu bar RAM glyph at 12x8, and kept tests passing.
- **Current focus:** Ready for user review.
- **Next action:** User can review the dashboard/menu bar and decide whether to close the stream.
- **Blockers:** none

## Progress log
- 2026-05-04 10:30 - Restored missing stream file and scoped requested display bugs.
- 2026-05-04 10:45 - Implemented display fixes and verified unit/compile checks.
- 2026-05-04 10:55 - Matched agentboard banner style and reduced menu bar icon width after visual review.
- 2026-05-04 11:00 - Switched to a distinct compact startup badge and scaled the menu bar glyph down again.
- 2026-05-04 11:05 - Restored a fancy banner without copying agentboard's pixel-art style.

## Open questions
_Things blocked on user input. Remove when resolved._
