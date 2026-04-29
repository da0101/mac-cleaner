---
domain_id: dom-browser-tab-optimizer
slug: browser-tab-optimizer
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [background-ai-optimizer, memory-optimization, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-04-29
---

# browser-tab-optimizer

## What this domain does

Inspects open browser tabs and uses AI/local rules to identify duplicate, converter/downloader, stale, or likely unnecessary Chrome tabs that are consuming memory. Closing tabs is destructive session control and must be designed separately from safe RAM purge.

## Backend / source of truth

- `server.py` owns local APIs and dashboard actions.
- Browser tab inventory should come from AppleScript against Google Chrome only in the first version.
- AI should receive minimal tab metadata: window index, tab index, title, domain, URL host, active/frontmost status when available.

## Frontend / clients

- Dashboard should show tab recommendations with title/domain, reason, risk, and explicit close controls.
- Terminal logs may summarize tab optimization opportunities, but dashboard confirmation is the primary UX.

## API contract locked

- The server must not accept arbitrary URLs or shell commands from the browser.
- Tab closing must target a fresh local snapshot; prefer stable Chrome tab IDs, and require title/domain/url agreement when falling back to window/tab indices.
- Closing tabs requires confirmation unless the user later explicitly enables a more aggressive mode.
- URLs/titles are local/private data and must not be logged verbosely.

## Key files

- `server.py`
- `ai_advisor.py`
- `tests/`

## Decisions locked

- First browser target is Google Chrome.
- Tab actions are opt-in and confirmation-gated.
- Last-visited age is best-effort only; open Chrome tabs do not expose reliable per-tab last-used time through AppleScript.
