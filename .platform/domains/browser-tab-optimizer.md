---
domain_id: dom-browser-tab-optimizer
slug: browser-tab-optimizer
status: retired
repo_ids: [mac-cleaner]
related_domain_slugs: [background-ai-optimizer, memory-optimization, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# browser-tab-optimizer

<!-- updated: 2026-11-16 — retired during removal of the Python/dashboard product surface. -->

## What this domain does

Historically inspected Chrome tabs and suggested close actions. Browser automation and tab closing are absent from the Swift/Rust product.

## Current contract

- Do not add AppleScript, browser metadata collection, or tab-closing authority as part of storage/memory cleanup.
- Reintroduction requires a separate privacy and confirmation design.

## Historical frontend / clients

- The removed dashboard showed tab recommendations with title/domain, reason, risk, and explicit close controls.
- The removed terminal flow summarized tab opportunities; none of this is a current native-app surface.

## Historical safety lessons retained

- Any future client must not accept arbitrary URLs or shell commands.
- Any future tab closing must target a fresh local snapshot and require explicit confirmation.
- URLs/titles are local/private data and must not be logged verbosely.

## Key files

- Historical implementation is available through Git history only.

## Decisions locked

- First browser target is Google Chrome.
- Tab actions are opt-in and confirmation-gated.
- Last-visited age is best-effort only; open Chrome tabs do not expose reliable per-tab last-used time through AppleScript.
