# mac-cleaner

Local macOS cleanup and RAM visibility tool for developers.

mac-cleaner scans known cache/log/development artifact locations, reports large macOS System Data contributors, and shows RAM/process pressure without automatically killing active work.

## Safety Model

mac-cleaner is intentionally conservative:

- Known cache/log/dev artifact paths can be cleaned.
- Mixed System Data folders are report-only unless a child path is explicitly classified safe.
- User documents, source repositories, credentials, app databases, keychains, mail, photos, and unknown data are never default-deleted.
- VS Code, Cursor, terminals, Claude Code, Codex, Gemini, Node, and Python jobs are protected from automatic termination.
- The dashboard binds to `127.0.0.1`.

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

## Usage

Terminal scanner:

```bash
./clean
```

Local dashboard:

```bash
./start
```

Then open:

```text
http://localhost:3333
```

Experimental AI assistant:

```bash
cp .env.example .env
./ai
```

## What It Scans

- `~/Library/Caches`
- user/system logs
- browser caches
- npm/Yarn/pnpm/pip/Homebrew/Gradle/Dart/CocoaPods caches
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
python3 -m py_compile scanner.py cleaner.py server.py ai_cleaner.py
```

## Release Status

`v1.0.0` is the first public release candidate. The dashboard and CLI scanner are useful; the AI assistant is still experimental and currently OpenAI-based.

## License

MIT
