---
stream_id: stream-pub-cache-protection
slug: pub-cache-protection
type: bug
status: closure-review
agent_owner: codex
domain_slugs: [storage-cleanup]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: main
created_at: 2026-04-30
updated_at: 2026-04-30
closure_approved: false
---

# pub-cache-protection

## Scope
- Protect Dart/Flutter's global Pub package cache from mac-cleaner deletion.
- Keep the target visible for storage reporting so users can understand disk usage.
- Add a regression test that proves `.pub-cache` is not cleanable and cannot be deleted through `clean_scan_item`.
- Out of scope: redesigning all package-store cleanup policy or restoring the user's Flutter cache.

## Done criteria
- [x] `~/.pub-cache` scan item is report-only/non-cleanable.
- [x] Direct cleanup calls refuse `.pub-cache` even if passed a stale cleanable item.
- [x] Relevant scanner tests pass with `python3 -m unittest tests.test_scanner`.
- [x] `.platform/memory/log.md` appended.
- [x] `decisions.md` updated if any architectural choices were made.

## Key decisions
2026-04-30 — Treat Pub cache as a protected development package store — deleting it can break active Flutter/Dart builds and should be done only through Pub/Flutter tooling.

## Resume state

- **Last updated:** 2026-04-30 — codex
- **What just happened:** Protected Pub cache in scanner policy, added hard cleanup guard, updated docs/memory, and verified tests.
- **Current focus:** Waiting for owner closure approval.
- **Next action:** If approved later, archive stream per closure protocol.
- **Blockers:** none

## Progress log
2026-04-30 00:00 — Verified `python3 -m unittest tests.test_scanner`, `python3 -m unittest`, and py_compile all pass.
2026-04-30 00:00 — Marked Pub cache report-only, added stale item deletion guard, and updated docs/memory.
2026-04-30 00:00 — Registered stream from user report that mac-cleaner wiped `~/.pub-cache`.

## Open questions
None.
