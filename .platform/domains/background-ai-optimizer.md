---
domain_id: dom-background-ai-optimizer
slug: background-ai-optimizer
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [ai-assistant, memory-optimization, storage-cleanup, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-04-29
---

# background-ai-optimizer

## What this domain does

Adds an AI decision layer that watches local RAM, processes, and cleanup findings in the background and recommends optimization actions. It may auto-run safe RAM purge in `--ai` mode, but destructive actions still require explicit user confirmation.

## Backend / source of truth

- `server.py` owns the local dashboard server, background monitor loops, and action endpoints.
- `scanner.py` owns storage target classification and cleanup safety tiers.
- AI recommendations must consume structured snapshots, not raw shell text.

## Frontend / clients

- Dashboard should surface AI recommendations, risk levels, expected savings, and action buttons.
- Terminal logs may show concise AI recommendation summaries, but the dashboard is the primary control surface.

## API contract locked

- AI may recommend `close`, `review`, `purge_ram`, or `clean_storage`; only `purge_ram` may run automatically, and it must not directly kill, quit, close, or delete without a separate confirmed action.
- Protected developer processes stay protected: VS Code, Cursor, Claude Code, Codex, Gemini, terminals, Node, Python, and active shell jobs.
- Storage cleanup uses scanner target IDs and safety tiers; browser/API requests must not accept arbitrary delete paths.

## Key files

- `server.py`
- `ai_advisor.py`
- `ai_cleaner.py`
- `scanner.py`
- `.env.example`

## Decisions locked

- Background AI is off by default; `--ai` enables Gemini recommendations plus automatic safe RAM purge only.
- Gemini is the target provider for new AI integration.
- OpenAI legacy chat can remain temporarily, but new background decisions should use a provider adapter.
