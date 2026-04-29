# mac-cleaner — Current Status

Last updated: 2026-04-29

> Local macOS storage and RAM optimizer for garbage files, caches, System Data visibility, and developer-machine hygiene.

## Feature areas

| Area | Status | Last touched | Notes |
|---|---|---|---|
| CLI storage scanner | 🔵 Exists | 2026-04-29 | `cleaner.py` scans known cache/log/dev artifact paths and prompts before cleanup. |
| Shared scanner | 🔵 Exists | 2026-04-29 | `scanner.py` centralizes targets, safety tiers, allocated disk sizing, and report-only System Data discovery. |
| Local dashboard | 🔵 Exists | 2026-04-29 | `server.py` serves localhost dashboard/API on port 3333; should default to scan-only for public release. |
| RAM/process optimizer | 🔵 Exists | 2026-04-29 | Shows available RAM, top process groups, and protected developer tools. |
| AI assistant | ⚠ Flagged | 2026-04-29 | `ai_cleaner.py` still uses OpenAI; Gemini migration is planned follow-up. |
| Open-source packaging | ✓ Done | 2026-04-29 | README/license/gitignore/CODEOWNERS added; `main` and `v1.0.0` pushed. Branch protection still needs GitHub settings. |

## Immediate priorities

1. **Gemini migration** — replace OpenAI assistant path with Gemini or make providers explicit.
2. **Installer scripts** — add user-local symlink install/uninstall scripts for `mac-cleaner`.
3. **Server refactor** — split `server.py` before adding major features.

## Known gotchas

- Developer tools and terminal sessions must never be killed automatically.
- System Data is a report-only discovery bucket unless a child path is explicitly classified safe.
- Sparse VM/container files must use allocated disk usage, not apparent size.
