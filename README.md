# Mac Cleaner 2

Mac Cleaner is a conservative macOS storage and system monitor for developers. Version 2 combines a pure SwiftUI/AppKit menu-bar application with a Rust engine, a shared Rust CLI, and a deliberately small cleanup policy.

This branch is an alpha release candidate. Its implementation languages are Swift and Rust only: no Python, TypeScript, React, Tauri, Node, web view, pip, or virtual environment is part of the source or packaged application. The historical Python version on `main` remains a Git rollback reference until the release gates are complete.

## Safety model

Mac Cleaner never accepts a filesystem path from the interface or CLI cleanup executor. Rust discovers candidates from a compiled, versioned catalog and issues opaque candidate IDs. Every mutation follows this boundary:

1. Run a read-only scan.
2. Select exact candidate IDs.
3. Review a short-lived plan with every path, size, reason, and recovery method.
4. Confirm the exact plan in a native dialog or by typing its UUID in the CLI.
5. Revalidate device, inode, type, size, mtime, link count, tree size, mount boundary, and policy immediately before moving anything.
6. Move the item to private quarantine and journal the result.
7. Restore it, or permanently purge one exact quarantine entry after another confirmation.

Automatic storage deletion is empty and disabled. A quarantine move does not claim reclaimed space; only a verified purge does.

The following are permanently protected:

- source repositories, Git worktrees, `.git`, project manifests, and lockfiles;
- GitHub Desktop Application Support, preferences, authentication state, and Keychains;
- Flutter, Dart, Android, Xcode, Gradle, CocoaPods, and other SDK/toolchain state;
- `~/.pub-cache`, `~/.gradle`, credentials, databases, documents, photos, and unknown paths;
- developer processes including editors, terminals, agents, compilers, Flutter/Dart/Gradle, and active jobs.

Automatic memory recovery is pressure-based: every 30 seconds Mac Cleaner reports conservative available, reclaimable, compressed, and swapped memory; colors the menu-bar RAM status; and notifies on sustained warning or critical pressure. It ranks unprotected apps and can open Activity Monitor for a user-confirmed decision. The Rust product has no process-signal, force-quit, fake RAM-purge, pressure-generator, DNS-flush, Docker-prune, Chrome-close, or unattended storage-cleanup API.

## Initial cleanup catalog

The first catalog only reviews direct, old children of three exact roots:

- Mac Cleaner's own cache, older than 24 hours;
- Xcode DerivedData children, older than 14 days;
- CocoaPods download-cache children, older than 30 days.

Entries containing project sentinels, symlinks, hard-linked files, special objects, mount crossings, protected paths, or too many descendants are skipped and explained. Mac Cleaner does not sweep `~/Library/Caches`, Application Support, Trash, package stores, or source folders.

## Architecture

- `cleaner-domain` — typed IDs, plans, receipts, quarantine actions, and snapshots.
- `cleaner-policy` — compiled allowlist and permanent protections.
- `cleaner-fs` — bounded scanner and identity-revalidated filesystem actions.
- `cleaner-system-macos` — read-only CPU, memory, disk, and process snapshots.
- `cleaner-store` — SQLite WAL settings, migration, plans, events, and receipts.
- `cleaner-service` — single-flight orchestration and cross-process mutation lock.
- `mac-cleaner-engine` — versioned one-shot JSON protocol used by the native app through fixed commands and opaque IDs.
- `mac-cleaner-mcp` — bundled closed-world MCP stdio server for local Codex and Claude diagnosis and confirmed cleanup.
- `mac-cleaner-macos` — pure SwiftUI/AppKit lifecycle, menu bar, native confirmations, and Xcode project.
- `mac-cleaner` — Rust CLI linked to the same service and policy.

The installed app embeds the Rust engine under `Contents/Helpers` and opens no HTTP listener. Swift resolves that helper through the application bundle and can invoke only a fixed command allowlist; it never supplies arbitrary cleanup paths or shell fragments.

## Claude and Codex MCP

The installed application also embeds `Contents/Helpers/mac-cleaner-mcp`, a local stdio server backed by the same Rust service and SQLite journal as the UI. It exposes closed-world inspection plus exact-plan cleanup tools:

- `latest_storage_report` — return the most recent saved path-level report without rescanning;
- `analyze_storage` — run a fresh measurement of the fixed developer-storage catalog;
- `system_health` — inspect current CPU, SSD, RAM, swap, pressure, and protected processes;
- `cleanup_candidates` — discover the conservative compiled cleanup catalog without changing files.
- `prepare_cleanup` / `execute_cleanup` — prepare an exact candidate-ID plan, then move explicitly confirmed items to Recovery;
- `prepare_tool_cleanup` / `execute_tool_cleanup` — prepare and confirm only `uv-cache` or verified old `simulator-recordings` cleanup;
- `list_recovery_items` — inspect recoverable items and their still-occupied space;
- `prepare_recovery_action` / `execute_recovery_action` — separately confirm one exact restore or permanent purge.

The MCP server accepts no path, shell text, Docker-prune target, repository, database, SDK, or runtime. Prepare calls change no files; every execution tool is marked destructive, requires the exact short-lived Rust plan and its plan-specific confirmation, revalidates identity, and records a receipt. General cleanup goes to Recovery first, and permanent reclamation is a separate single-item purge plan.

After installing the app, Settings → Claude & Codex access can copy the registration commands. The equivalent commands are:

```bash
codex mcp add mac-cleaner-storage -- "$HOME/Applications/Mac Cleaner.app/Contents/Helpers/mac-cleaner-mcp"
claude mcp add --scope user --transport stdio mac-cleaner-storage -- "$HOME/Applications/Mac Cleaner.app/Contents/Helpers/mac-cleaner-mcp"
```

Restart or begin a new agent session after registration so the host discovers the new tools.

## Developer setup

Requirements:

- macOS 13 or newer;
- Rust 1.96 with `aarch64-apple-darwin` and `x86_64-apple-darwin` targets;
- Xcode 16 or newer with macOS SDK and command-line tools.

```bash
./scripts/check-rust.sh
./scripts/dev-rust.sh
```

The developer build cannot register itself as a login item. Launch at login is available only from an installed `.app` bundle and requires a native confirmation.

## CLI

Read-only commands:

```bash
cargo run -p mac-cleaner -- status
cargo run -p mac-cleaner -- scan
cargo run -p mac-cleaner -- history
cargo run -p mac-cleaner -- quarantine list
```

Preview and quarantine exact IDs from a fresh scan:

```bash
cargo run -p mac-cleaner -- scan
cargo run -p mac-cleaner -- clean --candidate 'TARGET:HASH'
```

Restore or permanently purge one recorded quarantine entry:

```bash
cargo run -p mac-cleaner -- quarantine restore --entry UUID
cargo run -p mac-cleaner -- quarantine purge --entry UUID
```

Every mutating CLI command prints the exact action and requires its generated action UUID to be typed back. Candidate IDs and entry UUIDs are not paths.

## Legacy migration

Migration is explicit, copy-only, size-bounded, and idempotent. It imports only allowlisted display/monitor settings and sanitized cleanup summaries; retired destructive automation settings are ignored. Source files are preserved.

```bash
cargo run -p mac-cleaner -- migrate --from /absolute/path/to/legacy/mac-cleaner
```

The legacy Python LaunchAgent is not removed automatically. Startup cutover requires its own reviewed migration step so an installed Rust app and the Python menubar cannot silently compete.

## Packaging

`./scripts/build-universal.sh` builds a universal CLI, universal Rust engine, native SwiftUI application, and unsigned development DMG. Release distribution is a Developer ID signed and notarized DMG. Signing, notarization, updater credentials, Intel device QA, and upgrade/rollback verification remain release gates and are never bypassed for an alpha build.

## License

MIT
