# Feature Brief — mac-cleaner

**Feature:** Server refactor
**Status:** closure-review
**Stream file:** `work/server-refactor.md`

## What we're building

Break the 2,000+ line `server.py` monolith into focused modules while preserving the local dashboard, cleanup APIs, AI advisor, Chrome tab optimizer, and launcher behavior.

## Why

The current server file mixes state, settings, scanning, memory/process analysis, AI, Chrome tabs, HTML/JS/CSS, HTTP routing, and startup loops, making future changes risky.

## Done looks like

- `server.py` becomes a thin entrypoint/router surface.
- Related behavior moves into cohesive modules with tests updated.
- Existing commands and dashboard endpoints continue working.

## Relevant context

- `.platform/domains/server-refactor.md`
- `.platform/domains/local-dashboard.md`
- `.platform/domains/memory-optimization.md`
- `.platform/domains/storage-cleanup.md`
- `.platform/domains/background-ai-optimizer.md`
- `.platform/domains/browser-tab-optimizer.md`
- `.platform/conventions/security.md`
- `.platform/conventions/testing.md`
