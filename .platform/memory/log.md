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

2026-08-11 — pressure-based-memory-recovery — replaced legacy disk-cache purge and artificial-pressure generation with a 30-second Rust pressure/watchdog model, visible reclaimable/compressed/swap context, rate-limited alerts, ranked unprotected-process guidance, and a user-controlled Activity Monitor handoff; never terminate applications automatically.

2026-11-16 — native-swiftui-shell — replaced Tauri/React and the legacy Python product surface with a pure SwiftUI/AppKit app plus a fixed-command Rust JSON helper; added a modern accessible menu-bar popover and storage-history UI; strict Rust tests, Swift tests, live protocol QA, universal app/helper/CLI builds, and unsigned DMG packaging pass. Mac App Store sandbox/signing remains a separate release gate.

2026-11-16 — ab rescan — renewed Agentboard process files and rebuilt status, architecture, domain, security, testing, Rust, and Swift context around the verified native Swift/Rust product while marking the Python/web/AI domains retired.

2026-11-16 — agent-storage-mcp — bundled a universal inspection-only Rust MCP helper, registered it globally with Codex and at user scope with Claude, and verified both hosts retrieve the same 25-hotspot saved report; agent access never grants cleanup authority.

2026-11-16 — native-menubar-polish — replaced the monochrome status title with compact color-coded RAM and SSD labels and verified the final installed universal app in system/dark appearance; explicit text keeps the signal understandable without color.

2026-11-16 — actionable-native-workflow — turned passive Cleanup/Quarantine/Activity states into an actionable cleanup, recovery, and verification loop; exposed MCP as Agents; enabled persisted launch scans and last-tab state; and changed the app from accessory-only to a regular foreground Mac app. Safety gates and explicit confirmation remain unchanged.

2026-11-16 — cpu-and-agent-cleanup — exposed accurate total/per-process CPU in the native menu bar, popover, Overview, and MCP; expanded MCP from inspection to exact-plan confirmed cleanup/recovery without accepting paths or weakening repository, toolchain, runtime, or database protections.

2026-11-16 — semantic-menubar-status — moved the status label to a native AppKit status item so RAM, CPU, and SSD retain distinct pressure-colored SF Symbols; kept the SwiftUI popover and fixed the refresh spinner to replace its arrow inside one stable circle.

2026-11-16 — popover-brand-and-contrast — replaced the generic sparkle with a generated Mac Cleaner storage-health mark and made popover values, progress, captions, and process pressure readable against an opaque native surface.

- 2026-11-16 — commit `d625ab5`: feat: ship native Swift Mac Cleaner — auto-logged

- 2026-11-16 — commit `dd0dfa2`: feat: make cleanup actionable and hand off QA — auto-logged

- 2026-11-16 — commit `5f0a857`: chore: record cleanup handoff checkpoint — auto-logged

- 2026-11-17 — commit `085e760`: chore: remove legacy Python-era config, conventions, and retired domain docs — auto-logged

- 2026-11-17 — commit `8c16739`: chore: remove Python-era work streams — auto-logged

- 2026-11-17 — commit `b2562ec`: chore: drop Python-era streams from ACTIVE index — auto-logged

- 2026-11-17 — commit `901c278`: feat: offer project Xcode/iOS build output, Yarn project caches, device support and Chrome model for cleanup — auto-logged

- 2026-11-17 — commit `1d1508c`: fix: re-measure storage after a cleanup so removed items leave the managed list — auto-logged

- 2026-11-17 — commit `3acd0e0`: feat: auto-clean engine — opt-in RAM relief policy and scheduled storage cleanup — auto-logged

- 2026-11-17 — commit `8cb229a`: feat: Auto-Clean screen and scheduler in the native app — auto-logged

- 2026-11-17 — commit `7a28907`: fix: apply the protected-app filter when rows render so it follows the loaded list — auto-logged

- 2026-11-17 — commit `475f0f6`: fix: status bar no longer covers the Clean bar; Auto-Clean switches update instantly — auto-logged

- 2026-11-18 — commit `a0974ae`: feat: catalog Chrome, Homebrew, Playwright, Cypress, pip, Hugging Face, Bun, updater leftovers, XCTest clones and Maestro output — auto-logged
