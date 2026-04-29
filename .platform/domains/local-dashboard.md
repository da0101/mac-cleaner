---
domain_id: dom-local-dashboard
slug: local-dashboard
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [storage-cleanup, memory-optimization]
created_at: 2026-04-29
updated_at: 2026-04-29
---

# local-dashboard

Local browser dashboard for scan results, RAM/process summaries, history, and manual actions.

## Key files

- `server.py`
- `start`

## Locked contracts

- Bind to `127.0.0.1`.
- Public release should default to scan-only.
- Do not accept arbitrary deletion paths from browser requests.

