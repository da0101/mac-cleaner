# Session Log

2026-05-04 - menubar-memory-widget - aligned terminal, dashboard, and menu bar memory displays - raw free RAM and available RAM should stay distinct but visible together.

2026-04-30 — pub-cache-protection — protected Dart/Flutter Pub cache from scanner cleanup — package stores are not disposable caches during active development.

2026-04-29 — server-refactor — split `server.py` into `mac_cleaner_server/` modules and static dashboard assets — small entrypoint, behavior preserved.

2026-04-29 — background-ai-optimizer — added optional Gemini dashboard advisor with local validation — AI recommends, local code gates actions.

2026-04-29 — browser-tab-optimizer — added Chrome tab recommendations and confirmed close endpoint — never close active/unknown tabs.

2026-04-29 — open-source-release — restoring platform pack and preparing public release — hidden local runtime files must stay out of Git.

2026-04-29 — closed stream open-source-release → ./.platform/work/archive/open-source-release.md (by danilulmashev)

2026-04-29 — closed stream background-ai-optimizer → ./.platform/work/archive/background-ai-optimizer.md (by danilulmashev)

2026-04-29 — closed stream browser-tab-optimizer → ./.platform/work/archive/browser-tab-optimizer.md (by danilulmashev)

- 2026-04-29 — commit `8df1848`: Add AI optimizers and split dashboard server — auto-logged

- 2026-04-30 — commit `3a90ef6`: Protect Dart pub cache from cleanup — auto-logged

2026-05-04 — menubar-memory-widget — fixed auto-clean dashboard/widget sync — explicit set/status APIs prevent stale menu state from flipping the server incorrectly.

- 2026-05-04 — commit `7e0af3e`: Release v1.3.0 menu bar sync — auto-logged

2026-08-10 — full-rust-rewrite — implemented the Rust/Tauri application, CLI, fail-closed catalog, SQLite journal, read-only monitor, legacy JSON migration, recoverable quarantine, exact restore/purge, native startup gating, and universal macOS packaging; local gates and fake-home safety tests pass, while Developer ID signing/notarization and legacy LaunchAgent cutover remain release gates.

2026-08-10 — storage-pressure-diagnosis — replaced the Python widget with live Rust RAM/SSD status and added explanatory hotspot analysis — large known paths must be ranked by recovery semantics, never treated as garbage solely because of size.

2026-08-10 — emergency-storage-recovery — owner-approved `flutter clean` removed only inactive Astral generated output and restored about 10.7 GB; active Vibe work, Astral source, Git metadata, and manifests remained protected.

2026-08-10 — rust-memory-semantics — aligned Rust available RAM with the conservative reclaimable-page definition, separated completely-free RAM, and added accessible relative process/menu severity — raw free is context, available capacity and memory pressure drive decisions.

2026-08-10 — cleanup-entrypoint — moved reviewed cleanup beside storage diagnosis, added fixed owning-tool launch actions and a non-destructive RAM watchdog — tool-managed hotspots require journaled owning-tool plans before the Rust app may reclaim them.

2026-08-10 — owning-tool-audit — found 4.3 GB uv cache and one 2.2 GB shutdown simulator as the safest immediate reviewed targets; protected active Xcode/Simulator state and Docker volumes. `npm cache verify --dry-run` unexpectedly garbage-collected 169.6 MB and is now classified as mutating.

2026-08-10 — app-native-uv-prune — Mac Cleaner invoked uv's fixed built-in prune through an immutable confirmed plan and reclaimed 516.7 MB; persistent receipts and in-progress button state are required so successful cleanups never look inert or permit duplicate clicks.

2026-08-11 — simulator-recording-cleanup — owner-confirmed exact-file cleanup removed 453 old verified QuickTime/XCTest recordings and reclaimed 22.5 GB without deleting the simulator, apps, app data, source, or unrelated test results; large manifests require portable JSON integer types and tool receipts must share the visible activity trail.

2026-08-11 — rust-modularity-and-ui — split all entrypoint/orchestration files below 307 lines, fixed comprehensive scan feedback and per-tool busy state, and made the dashboard denser with text-backed green/amber/red/blue semantics; full Rust, Clippy, React, package, and installed-app gates pass.
