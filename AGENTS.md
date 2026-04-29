<!-- agentboard:root-entry:begin v=1 -->
# mac-cleaner

**What this is:** Local macOS cleanup scripts for finding garbage files, reclaiming storage, and optimizing RAM/process usage. The project must protect active development work while making cleanup opportunities visible and explainable.

## Stack

Standalone Python 3 scripts using macOS command-line tools, Python stdlib `http.server` for a localhost dashboard, JSON history storage, and a planned Gemini AI assistant via `google-genai`.

## Repo Structure

Single GitHub repository: `da0101/mac-cleaner`.

## How This Project Actually Works

- `./clean` runs `cleaner.py`, the terminal storage scanner/interactive cleaner.
- `./start` runs `server.py`, a localhost dashboard/API on `127.0.0.1:3333`.
- `./ai` runs `ai_cleaner.py`, currently OpenAI-based but slated for Gemini migration.
- `cleanup_history.json` stores recent dashboard cleanup summaries.
- Cleanup and process actions are local, powerful, and must be preview/confirmation-first.

## Workflow

Follow `.platform/workflow.md`: triage, interview, research, propose, execute, verify. New non-trivial work must create a tracked stream in `.platform/work/ACTIVE.md` before implementation.

## Reference Pack (.platform/)

- `.platform/STATUS.md` — current feature areas, priorities, blocklist, gotchas.
- `.platform/architecture.md` — components, data flow, invariants, debt.
- `.platform/repos.md` — single-repo routing and conventions map.
- `.platform/work/BRIEF.md` — current priority: open-source release prep.
- `.platform/domains/storage-cleanup.md`
- `.platform/domains/memory-optimization.md`
- `.platform/domains/ai-assistant.md`
- `.platform/domains/local-dashboard.md`
- `.platform/conventions/` — Python, security, testing, API, QA, deployment, permissions.
- `.platform/memory/` — decisions, gotchas, playbook, open questions, log.

## Hard Constraints (Don't Break These)

1. Never delete user documents, photos, source repos, credentials, app databases, keychains, or unknown data by default.
2. Every destructive cleanup action must be previewable and explain the target path before deletion.
3. Never close or kill VS Code, Claude Code, Codex, Gemini, terminal shells, or active developer jobs without explicit user confirmation.
4. Prefer memory pressure, soft reclamation, and app suggestions before force-killing processes.
5. Keep dashboard cleanup controls bound to `127.0.0.1`; do not expose them on the network.
6. Do not read or log `.env` contents or secrets.
7. Migrate AI behavior from OpenAI to Gemini using the official Google GenAI SDK.
<!-- agentboard:root-entry:end v=1 -->
