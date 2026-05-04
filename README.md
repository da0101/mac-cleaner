# mac-cleaner

Local macOS cleanup and RAM visibility tool for developers.

mac-cleaner scans known cache/log/development artifact locations, reports large macOS System Data contributors, and shows RAM/process pressure without automatically killing active work.

## What's New in v1.2.1

- Bug fix: terminal startup restored a branded Mac Cleaner splash without the internal server label.
- Bug fix: dashboard memory now matches the menu bar by showing raw free RAM and available RAM together.
- Bug fix: the menu bar RAM module is vertical, wider, and more visible, with corrected MB/GB labels.
- `v1.2.0` added the macOS menu bar RAM widget with free/available memory status.
- Bug fix: Dart/Flutter's global Pub package store (`~/.pub-cache`) is now report-only protected storage and cleanup refuses to delete it.
- Gemini-backed dashboard recommendations with `--ai` for automatic safe RAM purge and `--ai-advisory` for recommendations only.
- Chrome Tab Optimizer that inspects local Chrome tab metadata, ranks likely duplicate/media/converter tabs, and requires confirmation before closing.
- Dashboard settings modal with persisted `settings.json` controls for refresh intervals, target available RAM, auto-clean timing, visible widgets, and dark/light/system theme.
- Primary cleanup controls now stay at the top of the dashboard for fast scan, clean, Docker prune, RAM purge, and auto-clean toggling.
- The 2,000+ line dashboard server was split into the `mac_cleaner_server/` package so future cleanup, memory, AI, Chrome, and UI work is easier to maintain.
- macOS memory reporting now emphasizes available/reusable RAM instead of raw free pages.

## Safety Model

mac-cleaner is intentionally conservative:

- Known cache/log/dev artifact paths can be cleaned.
- Mixed System Data folders are report-only unless a child path is explicitly classified safe.
- User documents, source repositories, credentials, app databases, keychains, mail, photos, and unknown data are never default-deleted.
- VS Code, Cursor, terminals, Claude Code, Codex, Gemini, Node, and Python jobs are protected from automatic termination.
- The dashboard binds to `127.0.0.1`.
- Chrome tab optimization inspects local tab titles/domains and asks before closing any tab.

## Requirements

- macOS
- Python 3.10+
- Optional: Docker CLI for Docker storage reporting/pruning
- Optional: OpenAI/Gemini API keys for the experimental AI assistant

Install Python dependencies:

```bash
python3 -m venv .venv
. .venv/bin/activate
python -m pip install -r requirements.txt
```

Install the `mac-cleaner` command:

```bash
./install
```

## Usage

Terminal scanner:

```bash
./clean
```

Local dashboard:

```bash
./start
# or, after ./install:
mac-cleaner
```

Local dashboard with Gemini AI and automatic safe RAM optimization:

```bash
cp .env.example .env
# add GEMINI_API_KEY to .env
./start --ai
# or:
mac-cleaner --ai
```

Chrome tab optimizer:

```text
Open the dashboard and use the Chrome Tab Optimizer panel.
Recommended tabs require confirmation before closing.
```

Gemini recommendations only, without automatic RAM purge:

```bash
./start --ai-advisory
# or:
mac-cleaner --ai-advisory
```

Run without background AI:

```bash
./start --no-ai
# or:
mac-cleaner --no-ai
```

Then open:

```text
http://localhost:3333
```

Legacy terminal AI assistant:

```bash
cp .env.example .env
./ai
```

## What It Scans

- `~/Library/Caches`
- user/system logs
- browser caches
- npm/Yarn/pnpm/pip/Homebrew/Gradle/CocoaPods caches
- Dart/Flutter Pub package store as report-only protected storage
- Xcode DerivedData, archives, iOS Device Support, simulator caches
- VS Code/Cursor caches and logs
- Slack/Teams/Spotify caches
- Trash
- AI/ML caches
- Docker storage
- report-only System Data areas like `~/Library/Application Support`, `~/Library/Containers`, `/private/var/folders`, `/private/var/vm`, and `~/.cache`

## Development

Run tests:

```bash
python3 -m unittest
python3 -m py_compile scanner.py cleaner.py server.py ai_cleaner.py ai_advisor.py browser_tabs.py mac_cleaner_server/*.py
```

The local dashboard server now lives in focused modules under `mac_cleaner_server/`; keep `server.py` as the thin executable entrypoint.

## Release Status

`v1.2.1` fixes the terminal startup splash, dashboard memory card, and menu bar RAM widget visuals. `v1.2.0` added the macOS menu bar RAM widget. `v1.1.1` protects Dart/Flutter's `~/.pub-cache` package store from cleanup. `v1.1.0` added the background Gemini optimizer, Chrome tab recommendations, persistent dashboard settings, light/dark mode, and the split server package. Destructive storage cleanup and Chrome tab closing remain confirmation-based. The separate `./ai` terminal assistant is still legacy OpenAI-based.

## License

MIT
