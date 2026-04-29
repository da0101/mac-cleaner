---
domain_id: dom-memory-optimization
slug: memory-optimization
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [ai-assistant, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-04-29
---

# memory-optimization

Reports available/reusable RAM and groups heavy processes while protecting active developer work.

## Key files

- `server.py`
- `ai_cleaner.py`

## Locked contracts

- Use available/reusable RAM for health, not raw free pages.
- Do not auto-kill developer tools or shell jobs.
- VS Code, Cursor, Claude Code, Codex, Gemini, terminals, Node, and Python jobs are protected.

