---
domain_id: dom-rust-application
slug: rust-application
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [safe-cleanup-policy, storage-cleanup, memory-optimization, local-dashboard]
created_at: 2026-08-10
updated_at: 2026-08-11
---

# rust-application

Full replacement of the Python cleaner, localhost dashboard, menu-bar widget, and installers with a release-grade Rust macOS application and CLI.

## Existing implementation to replace

- `scanner.py`, `cleaner.py` — scan policy, preview, and destructive execution.
- `server.py`, `mac_cleaner_server/` — localhost API, scheduling, memory, and process monitoring.
- `mac_cleaner_server/assets/` — dashboard interface.
- `menubar.py`, `menubar-install`, `menubar-uninstall` — login item and menu bar.
- `ai_advisor.py`, `browser_tabs.py`, `ai_cleaner.py` — recommendations, Chrome inspection, and legacy assistant.
- `install`, `uninstall`, `start`, `clean`, `ai`, `requirements.txt` — Python distribution surface.

## Replacement boundaries

- Rust owns scanning, policy, cleanup plans, execution, system metrics, persistence, CLI, app lifecycle, and packaging.
- The shipped product requires no Python, pip, virtual environment, or Node runtime.
- Tauri hosts bundled local UI assets; the frontend receives no filesystem, shell, process, or network authority.
- The application uses typed Tauri IPC rather than a localhost HTTP listener.
- The safe-cleanup policy is a release-blocking contract.
- The menu-bar title is read-only and shows compact available RAM and SSD values from the shared Rust system snapshot.
- Storage-pressure analysis runs only on request, measures a fixed catalog plus manifest-recognized generated project leaves, and labels each result as reviewable, tool-managed, or protected. Its report never grants cleanup authority.
- Every recognized category follows the same reusable workflow: analyze, explain, preview a service-owned plan, confirm natively, revalidate, execute, and issue a receipt.
- RAM optimization reports conservative available, reclaimable, compressed, file-backed, and swapped memory. The 30-second watchdog derives normal/warning/critical pressure, notifies on sustained pressure, ranks unprotected apps, and opens Activity Monitor for user-confirmed recovery. It does not manufacture “free” RAM by flushing disk buffers or generating pressure.

## Locked contracts

- No arbitrary path deletion; only versioned catalog IDs and fresh service-owned plans may execute.
- Automatic storage cleanup starts with an empty allowlist.
- Repositories, Git metadata, GitHub Desktop state, Flutter/Dart/Gradle/CocoaPods state, SDKs, credentials, databases, keychains, documents, photos, and unknown paths are protected.
- Every reviewed cleanup shows exact paths, impact, recovery method, and confirmation before mutation.
- Developer tools and active jobs are never killed, signaled, closed, or manipulated automatically.
- Login startup, upgrades, crash recovery, uninstall, universal binaries, signing, notarization, and rollback are release requirements.
- Python remains only as a migration reference until the Rust release candidate passes safety and parity gates.

## Compatibility surfaces

- Existing `settings.json` and `cleanup_history.json` require a copy-only, versioned import.
- Existing command intents need explicit parity or owner-approved retirement.
- Existing dashboard workflows need behavioral parity even though the server architecture is retired.
