# Mac Cleaner 2

Mac Cleaner is a conservative macOS storage and system monitor for developers. Version 2 is a Rust rewrite with a native Tauri menu-bar app, a shared Rust CLI, and a deliberately small cleanup policy.

This branch is an alpha release candidate. It does not require Python, pip, a virtual environment, or a Node runtime after packaging. The Python version on `main` remains the rollback reference until the release gates are complete.

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

The process monitor is read-only. The Rust product has no process-signal, force-quit, RAM-purge, DNS-flush, Docker-prune, Chrome-close, or unattended storage-cleanup API.

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
- `cleaner-system-macos` — read-only memory, disk, and process snapshots.
- `cleaner-store` — SQLite WAL settings, migration, plans, events, and receipts.
- `cleaner-service` — single-flight orchestration and cross-process mutation lock.
- `mac-cleaner-desktop` — Tauri 2 lifecycle, tray, native confirmations, and bundled UI.
- `mac-cleaner` — Rust CLI linked to the same service and policy.

The installed app bundles its interface and opens no HTTP listener. Vite binds to `127.0.0.1:1420` only during development. The frontend has no filesystem, shell, process, or arbitrary network capability; it can call only the registered Tauri commands.

## Developer setup

Requirements:

- macOS 13 or newer;
- Rust 1.96 with `aarch64-apple-darwin` and `x86_64-apple-darwin` targets;
- Node 22.12 or newer for UI build tooling only;
- Xcode command-line tools.

```bash
npm ci --prefix apps/mac-cleaner-desktop/ui
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

`./scripts/build-universal.sh` builds a universal CLI and universal Tauri app. Release distribution is a Developer ID signed and notarized DMG. Signing, notarization, updater credentials, Intel device QA, and upgrade/rollback verification remain release gates and are never bypassed for an alpha build.

## License

MIT
