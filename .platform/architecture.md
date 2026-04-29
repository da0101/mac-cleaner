# mac-cleaner — Architecture

Last updated: 2026-04-29

## What this system does

mac-cleaner is a local-only macOS cleanup tool. It finds safe-to-delete caches/logs/dev artifacts, reports large System Data contributors, shows RAM availability, and surfaces heavy processes without interrupting active developer work.

## Components

```text
./clean  -> cleaner.py     -> terminal scan + interactive cleanup
./start  -> server.py      -> thin entrypoint for localhost dashboard/API on 127.0.0.1:3333; optional `--ai`
./ai     -> ai_cleaner.py  -> legacy OpenAI terminal assistant
scanner.py                -> shared storage target catalog + safety tiers
ai_advisor.py             -> Gemini structured recommendations + local safety validation
browser_tabs.py           -> Chrome tab inventory, local tab recommendations, and safe close helper
mac_cleaner_server/       -> dashboard server modules, state, routing, services, and static assets
tests/                    -> unittest scanner coverage
```

## Stack

| Layer | Choice |
|---|---|
| Language | Python 3 |
| Web server | Python stdlib `http.server` |
| State | Local JSON cleanup history |
| AI | Optional Gemini dashboard advisor; OpenAI legacy terminal chat |
| Distribution | Local scripts / open-source repo |

## Invariants

1. Never delete user documents, photos, source repos, credentials, app databases, keychains, or unknown data by default.
2. Destructive cleanup must be preview/confirmation-first.
3. Dashboard cleanup controls bind to `127.0.0.1`.
4. Developer tools, shells, Codex, Claude, Gemini, Node, and Python jobs are protected unless the user explicitly approves interruption.
5. System Data discovery is path-level and report-only unless a child path is classified safe.
6. Secrets live in `.env` and are never committed, printed, or logged.
7. Background AI may auto-run safe RAM purge only; local code validates recommendations and confirmations before any close/delete actions.
8. Chrome tab closing uses fresh snapshot validation and per-tab confirmation.

## Known debt

| Area | Issue | Planned fix |
|---|---|---|
| `ai_cleaner.py` | OpenAI-specific and separate from background advisor. | Migrate to Gemini adapter or remove. |
