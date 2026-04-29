# mac-cleaner — Architecture

Last updated: 2026-04-29

## What this system does

mac-cleaner is a local-only macOS cleanup tool. It finds safe-to-delete caches/logs/dev artifacts, reports large System Data contributors, shows RAM availability, and surfaces heavy processes without interrupting active developer work.

## Components

```text
./clean  -> cleaner.py     -> terminal scan + interactive cleanup
./start  -> server.py      -> localhost dashboard/API on 127.0.0.1:3333
./ai     -> ai_cleaner.py  -> legacy OpenAI terminal assistant
scanner.py                -> shared storage target catalog + safety tiers
tests/                    -> unittest scanner coverage
```

## Stack

| Layer | Choice |
|---|---|
| Language | Python 3 |
| Web server | Python stdlib `http.server` |
| State | Local JSON cleanup history |
| AI | OpenAI legacy; Gemini planned |
| Distribution | Local scripts / open-source repo |

## Invariants

1. Never delete user documents, photos, source repos, credentials, app databases, keychains, or unknown data by default.
2. Destructive cleanup must be preview/confirmation-first.
3. Dashboard cleanup controls bind to `127.0.0.1`.
4. Developer tools, shells, Codex, Claude, Gemini, Node, and Python jobs are protected unless the user explicitly approves interruption.
5. System Data discovery is path-level and report-only unless a child path is classified safe.
6. Secrets live in `.env` and are never committed, printed, or logged.

## Known debt

| Area | Issue | Planned fix |
|---|---|---|
| `server.py` | Large mixed file: backend, scheduler, dashboard HTML/JS. | Split into modules. |
| `ai_cleaner.py` | OpenAI-specific and provider logic inline. | Migrate to Gemini adapter. |
| Dashboard cleanup | Needs stricter UX confirmation for public users. | Default scan-only and explicit opt-in. |

