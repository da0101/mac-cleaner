---
domain_id: dom-rust-application
slug: rust-application
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [safe-cleanup-policy, storage-cleanup, memory-optimization, native-macos-shell, storage-forensics]
created_at: 2026-08-10
updated_at: 2026-11-16
---

# rust-application

Rust backend, CLI, persistence, policy, and native-app protocol for the SwiftUI macOS product. The Python cleaner, localhost dashboard, and web menu-bar widget have been fully removed from this branch.

## Retired implementation context

<!-- updated: 2026-11-16 — all listed v1 files were removed; names remain as historical migration context only. -->

- `scanner.py`, `cleaner.py` — scan policy, preview, and destructive execution.
- `server.py`, `mac_cleaner_server/` — localhost API, scheduling, memory, and process monitoring.
- `mac_cleaner_server/assets/` — dashboard interface.
- `menubar.py`, `menubar-install`, `menubar-uninstall` — login item and menu bar.
- `ai_advisor.py`, `browser_tabs.py`, `ai_cleaner.py` — recommendations, Chrome inspection, and legacy assistant.
- `install`, `uninstall`, `start`, `clean`, `ai`, `requirements.txt` — Python distribution surface.

## Replacement boundaries

- Rust owns scanning, policy, cleanup plans, execution, system metrics, persistence, the CLI, and the constrained native-app engine protocol.
- The shipped product requires no Python, pip, virtual environment, or Node runtime.
- SwiftUI/AppKit owns native presentation, application lifecycle, menu bar, notifications, and login-item preferences.
- The application uses a bundled one-shot Rust helper with a versioned JSON protocol rather than a localhost HTTP listener or web view.
- The application also bundles a separate Rust stdio MCP helper so Codex and Claude can inspect CPU/storage evidence and invoke the same closed-world cleanup service. It accepts opaque IDs rather than paths and separates non-destructive preparation from explicitly confirmed destructive execution.
- The safe-cleanup policy is a release-blocking contract.
- The menu-bar title is read-only and shows compact severity-coded RAM, CPU, and SSD values from the shared Rust system snapshot; its popover lists the top sampled CPU consumers without terminating them.
- Storage-pressure analysis runs only on request, measures a fixed catalog plus manifest-recognized generated project leaves, and labels each result as reviewable, tool-managed, or protected. Its report never grants cleanup authority.
- Every recognized category follows the same reusable workflow: analyze, explain, preview a service-owned plan, confirm natively, revalidate, execute, and issue a receipt.
- System monitoring reports CPU load, top CPU/memory jobs, conservative available/reclaimable/compressed/file-backed/swapped memory, and pressure every five seconds. It notifies on sustained memory pressure and opens Activity Monitor for user-confirmed recovery; it never terminates jobs, flushes disk buffers, or manufactures pressure.

## Locked contracts

- No arbitrary path deletion; only versioned catalog IDs and fresh service-owned plans may execute.
- Automatic storage cleanup starts with an empty allowlist.
- Repositories, Git metadata, GitHub Desktop state, Flutter/Dart/Gradle/CocoaPods state, SDKs, credentials, databases, keychains, documents, photos, and unknown paths are protected.
- Every reviewed cleanup shows exact paths, impact, recovery method, and confirmation before mutation.
- Developer tools and active jobs are never killed, signaled, closed, or manipulated automatically.
- Login startup, upgrades, crash recovery, uninstall, universal binaries, signing, notarization, and rollback are release requirements.
- Historical Python behavior remains available through Git history only; no Python source or runtime ships on this branch.

## Compatibility surfaces

- Existing `settings.json` and `cleanup_history.json` require a copy-only, versioned import.
- Existing command intents need explicit parity or owner-approved retirement.
- Historical dashboard workflows were either mapped to native views or explicitly retired; no localhost compatibility surface remains.
