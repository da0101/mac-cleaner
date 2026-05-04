---
domain_id: dom-local-dashboard
slug: local-dashboard
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [storage-cleanup, memory-optimization]
created_at: 2026-04-29
updated_at: 2026-05-04
---

# local-dashboard

Local browser dashboard for scan results, RAM/process summaries, history, and manual actions.

## Key files

- `server.py` is the thin executable entrypoint.
- `start` launches the local dashboard command from anywhere.
- `mac_cleaner_server/runtime.py` owns startup banner, server lifecycle, and background threads.
- `mac_cleaner_server/http_api.py` serves dashboard APIs on localhost.
- `mac_cleaner_server/assets/dashboard.html`
- `mac_cleaner_server/assets/dashboard.css`
- `mac_cleaner_server/assets/dashboard_actions.js`
- `menubar.py` mirrors core dashboard stats/actions in the macOS menu bar dropdown.

## Locked contracts

- Bind to `127.0.0.1`.
- Public release should default to scan-only.
- Do not accept arbitrary deletion paths from browser requests.
- Memory display should keep free RAM and available/reusable RAM visible as distinct values.
- Menu bar cleanup actions must keep the same confirmation-first behavior as dashboard cleanup controls.
- Auto-clean countdowns are server-owned (`next_ram_purge`, `next_garbage_clean`) and exposed through `/api/status` plus lightweight `/api/auto-clean-status`; browser and menu bar UIs should not maintain independent schedules.
- Use explicit `/api/auto-clean` set requests for widget/dashboard controls when possible; avoid stale UI state blindly flipping `/api/toggle-auto`.
