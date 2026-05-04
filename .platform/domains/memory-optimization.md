---
domain_id: dom-memory-optimization
slug: memory-optimization
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [ai-assistant, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-05-04
---

# memory-optimization

Reports available/reusable RAM and groups heavy processes while protecting active developer work.

## Key files

- `mac_cleaner_server/system.py` parses macOS RAM and disk metrics.
- `mac_cleaner_server/processes.py` builds RAM/process summaries.
- `mac_cleaner_server/memory.py` handles RAM purge and watchdog behavior.
- `mac_cleaner_server/assets/dashboard_actions.js` renders the dashboard memory stat card.
- `menubar.py` renders the macOS menu bar free/available RAM widget.
- `ai_cleaner.py` is the legacy terminal AI memory assistant.

## Locked contracts

- Use available/reusable RAM for health, not raw free pages.
- Show raw free RAM alongside available/reusable RAM when comparing with the menu bar widget.
- Do not auto-kill developer tools or shell jobs.
- VS Code, Cursor, Claude Code, Codex, Gemini, terminals, Node, and Python jobs are protected.
