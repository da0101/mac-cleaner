# mac-cleaner — Current Status

Last updated: 2026-04-29

> Local macOS storage and RAM optimizer for garbage files, caches, System Data visibility, and developer-machine hygiene.

## Feature areas

| Area | Status | Last touched | Notes |
|---|---|---|---|
| CLI storage scanner | 🔵 Exists | 2026-04-29 | `cleaner.py` scans known cache/log/dev artifact paths and prompts before cleanup. |
| Shared scanner | 🔵 Exists | 2026-04-29 | `scanner.py` centralizes targets, safety tiers, allocated disk sizing, and report-only System Data discovery. |
| Local dashboard | 🔵 Exists | 2026-04-29 | `server.py` is a thin entrypoint; `mac_cleaner_server/` serves localhost dashboard/API on port 3333 and defaults to scan-only. |
| RAM/process optimizer | 🔵 Exists | 2026-04-29 | Shows available RAM, top process groups, and protected developer tools. |
| Background AI optimizer | ✓ Done | 2026-04-29 | Optional `./start --ai` Gemini advisor shows dashboard recommendations and may auto-run safe RAM purge only; close/delete actions remain confirmation-gated. |
| Browser tab optimizer | ✓ Done | 2026-04-29 | Chrome tab recommendations inspect local tab metadata, including converter/downloader candidates; closing a tab requires confirmation and fresh snapshot validation. |
| AI assistant | ⚠ Flagged | 2026-04-29 | `ai_cleaner.py` remains legacy OpenAI terminal chat; background advisor uses Gemini separately. |
| Open-source packaging | ✓ Done | 2026-04-29 | README/license/gitignore/CODEOWNERS added; `main` and `v1.0.0` pushed. Branch protection still needs GitHub settings. |

## Immediate priorities

1. **Legacy AI assistant cleanup** — migrate or remove the separate OpenAI terminal chat.

## Known gotchas

- Developer tools and terminal sessions must never be killed automatically.
- System Data is a report-only discovery bucket unless a child path is explicitly classified safe.
- Sparse VM/container files must use allocated disk usage, not apparent size.
