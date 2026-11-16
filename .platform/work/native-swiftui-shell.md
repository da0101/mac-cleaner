---
stream_id: stream-native-swiftui-shell
slug: native-swiftui-shell
type: refactor
status: in-progress
agent_owner: codex
domain_slugs: [native-macos-shell, rust-application, safe-cleanup-policy, storage-forensics]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: feature/full-rust-rewrite
created_at: 2026-11-16
updated_at: 2026-11-16
closure_approved: false
---

# native-swiftui-shell

## Scope

- Replace the Tauri/React desktop shell with an Apple-native SwiftUI macOS application.
- Keep the Rust workspace as the backend and expose a constrained local JSON command boundary.
- Preserve monitoring, storage forensics, cleanup review/confirmation, quarantine, startup, and menu-bar behavior.
- Remove the web frontend and Tauri dependencies only after native parity and packaged-app verification.

## Done criteria

- [x] SwiftUI app builds and runs as a signed-capable macOS `.app` using Apple UI frameworks only.
- [x] Rust helper is embedded and addressed by a fixed bundle-relative path.
- [x] Native views cover system overview, storage forensics/history, cleanup review, quarantine, processes, settings, and activity.
- [x] Native confirmation and Rust policy gates remain intact for every destructive action.
- [x] Tauri/React shell and dependencies are removed after parity tests pass.
- [x] Rust, Swift, packaging, accessibility, and live macOS QA pass.
- [x] `.platform/memory/log.md` appended and architectural decisions recorded.

## Key decisions

2026-11-16 — Owner requires a pure macOS UI: SwiftUI/AppKit only, with Rust retained as the backend.

2026-11-16 — Recommend a bundled one-shot Rust engine with a versioned JSON protocol over C FFI — this preserves the workspace's unsafe-code prohibition, prevents memory-ownership bugs, and keeps Rust authoritative.

2026-11-16 — Resolve the helper from the fixed, symlink-validated `Contents/Helpers/mac-cleaner-engine` bundle path and expose fixed commands with opaque IDs — Swift never runs a shell, searches `PATH`, or supplies cleanup paths.

2026-11-16 — Keep the current shell until native parity is verified, then remove Tauri/React/Node atomically — this gives the migration a working rollback point.

2026-11-16 — Keep App Sandbox disabled for the development build until the storage-access model is redesigned with sandbox-safe user consent — silently enabling it now would break the core analysis contract. Mac App Store distribution remains a separate release gate.

## Resume state
_Overwritten by `ab checkpoint` — the compact payload the next agent reads first. Keep this block under ~10 lines._

- **Last updated:** 2026-11-16 by danilulmashev
- **What just happened:** Added live color-coded CPU status to the menu bar, a top-CPU popover list, a CPU overview card, and a fixed circular refresh control that swaps its glyph for the spinner without shifting.
- **Current focus:** Install and visually QA the CPU/menu refresh treatment alongside the expanded MCP workflow.
- **Next action:** Owner reviews the installed build; keep stream open until explicit closure approval.
- **Blockers:** none

## Progress log

2026-11-16 13:25 — Added a five-second CPU/RAM/SSD status refresh, accurate interval-based process CPU sampling, top CPU consumers in the menu popover, CPU health in Overview, and one stable circular toolbar refresh control whose arrow is replaced—not accompanied—by its spinner.

2026-11-16 13:13 — Reworked passive/empty tabs into a connected Diagnose → Cleanup → Recovery → History workflow, added visible Agents/MCP access, enabled default-on launch analysis, persisted last navigation, and removed `LSUIElement` so Mac Cleaner owns the macOS menu while focused. Full Rust/Swift tests, universal packaging, installed-app visual QA, and stable Codex/Claude MCP registration pass.

2026-11-16 12:10 — Final installed-app QA passed in dark/system appearance. The menu bar renders explicit severity-colored `RAM 4.0G` and `SSD 193G` values; the main window shows live storage, memory, swap, and protected-process data without the former gray slab or outer shadow.

2026-11-16 11:17 — Live QA caught and fixed packaged Rust helper discovery; the universal package now validates the exact helper path and protocol before succeeding. Rust/Swift tests, read-only helper views, universal architectures, DMG creation, Agentboard doctor, and recall checks pass. The rebuilt native app renders live data without the warning.
    diff feature/full-rust-rewrite vs main:
       .github/workflows/rust.yml                         |   49 +
       .gitignore                                         |    9 +
       .platform/domains/rust-application.md              |   50 +
       .platform/domains/safe-cleanup-policy.md           |   41 +
       .platform/memory/decisions.md                      |    6 +
       .platform/memory/learnings.md                      |   21 +
       .platform/memory/log.md                            |   20 +
       .platform/work/ACTIVE.md                           |    1 +
       .platform/work/BRIEF.md                            |   18 +-
       .platform/work/full-rust-rewrite.md                |  107 +
       .../work/qa/full-rust-rewrite-execution-journal.md |   22 +
       .platform/work/qa/full-rust-rewrite-manual-qa.md   |   35 +
       Cargo.lock                                         | 5851 ++++++++++++++++++++
       Cargo.toml                                         |   50 +
       README.md                                          |  167 +-
       apps/mac-cleaner-cli/Cargo.toml                    |   20 +
       apps/mac-cleaner-cli/src/app.rs                    |   34 +
       apps/mac-cleaner-cli/src/cli.rs                    |   90 +
       apps/mac-cleaner-cli/src/commands/cleanup.rs       |   33 +
       apps/mac-cleaner-cli/src/commands/migration.rs     |   29 +
       apps/mac-cleaner-cli/src/commands/mod.rs           |    4 +
       apps/mac-cleaner-cli/src/commands/quarantine.rs    |   59 +
       apps/mac-cleaner-cli/src/commands/reporting.rs     |   68 +
       apps/mac-cleaner-cli/src/confirmation.rs           |   14 +
       apps/mac-cleaner-cli/src/main.rs                   |   21 +
       apps/mac-cleaner-cli/src/output.rs                 |   61 +
       apps/mac-cleaner-desktop/app-icon.png              |  Bin 0 -> 2198531 bytes
       apps/mac-cleaner-desktop/src-tauri/Cargo.toml      |   34 +
       apps/mac-cleaner-desktop/src-tauri/build.rs        |    3 +
       .../src-tauri/capabilities/main.json               |    7 +
       .../src-tauri/gen/schemas/acl-manifests.json       |    1 +
       .../src-tauri/gen/schemas/capabilities.json        |    1 +
       .../src-tauri/gen/schemas/desktop-schema.json      | 2598 +++++++++
       .../src-tauri/gen/schemas/macOS-schema.json        | 2598 +++++++++
       .../src-tauri/icons/128x128.png                    |  Bin 0 -> 22806 bytes
       .../src-tauri/icons/128x128@2x.png                 |  Bin 0 -> 78300 bytes
       apps/mac-cleaner-desktop/src-tauri/icons/32x32.png |  Bin 0 -> 2367 bytes
       apps/mac-cleaner-desktop/src-tauri/icons/64x64.png |  Bin 0 -> 7177 bytes
       .../src-tauri/icons/Square107x107Logo.png          |  Bin 0 -> 16817 bytes
       .../src-tauri/icons/Square142x142Logo.png          |  Bin 0 -> 27157 bytes
       .../src-tauri/icons/Square150x150Logo.png          |  Bin 0 -> 29699 bytes
       .../src-tauri/icons/Square284x284Logo.png          |  Bin 0 -> 95417 bytes
       .../src-tauri/icons/Square30x30Logo.png            |  Bin 0 -> 2137 bytes
       .../src-tauri/icons/Square310x310Logo.png          |  Bin 0 -> 113603 bytes
       .../src-tauri/icons/Square44x44Logo.png            |  Bin 0 -> 3924 bytes
       .../src-tauri/icons/Square71x71Logo.png            |  Bin 0 -> 8554 bytes
       .../src-tauri/icons/Square89x89Logo.png            |  Bin 0 -> 12485 bytes
       .../src-tauri/icons/StoreLogo.png                  |  Bin 0 -> 4845 bytes
       .../android/mipmap-anydpi-v26/ic_launcher.xml      |    5 +
       .../icons/android/mipmap-hdpi/ic_launcher.png      |  Bin 0 -> 3740 bytes
       .../android/mipmap-hdpi/ic_launcher_foreground.png |  Bin 0 -> 34069 bytes
       .../android/mipmap-hdpi/ic_launcher_round.png      |  Bin 0 -> 3841 bytes
       .../icons/android/mipmap-mdpi/ic_launcher.png      |  Bin 0 -> 3637 bytes
       .../android/mipmap-mdpi/ic_launcher_foreground.png |  Bin 0 -> 17127 bytes
       .../android/mipmap-mdpi/ic_launcher_round.png      |  Bin 0 -> 3658 bytes
       .../icons/android/mipmap-xhdpi/ic_launcher.png     |  Bin 0 -> 11245 bytes
       .../mipmap-xhdpi/ic_launcher_foreground.png        |  Bin 0 -> 56875 bytes
       .../android/mipmap-xhdpi/ic_launcher_round.png     |  Bin 0 -> 11187 bytes
       .../icons/android/mipmap-xxhdpi/ic_launcher.png    |  Bin 0 -> 21887 bytes
       .../mipmap-xxhdpi/ic_launcher_foreground.png       |  Bin 0 -> 124139 bytes
       .../android/mipmap-xxhdpi/ic_launcher_round.png    |  Bin 0 -> 21878 bytes
       .../icons/android/mipmap-xxxhdpi/ic_launcher.png   |  Bin 0 -> 35657 bytes
       .../mipmap-xxxhdpi/ic_launcher_foreground.png      |  Bin 0 -> 228038 bytes
       .../android/mipmap-xxxhdpi/ic_launcher_round.png   |  Bin 0 -> 35441 bytes
       .../android/values/ic_launcher_background.xml      |    4 +
       apps/mac-cleaner-desktop/src-tauri/icons/icon.icns |  Bin 0 -> 2388823 bytes
       apps/mac-cleaner-desktop/src-tauri/icons/icon.ico  |  Bin 0 -> 95442 bytes
       apps/mac-cleaner-desktop/src-tauri/icons/icon.png  |  Bin 0 -> 330894 bytes
       .../src-tauri/icons/ios/AppIcon-20x20@1x.png       |  Bin 0 -> 1149 bytes
       .../src-tauri/icons/ios/AppIcon-20x20@2x-1.png     |  Bin 0 -> 3399 bytes
       .../src-tauri/icons/ios/AppIcon-20x20@2x.png       |  Bin 0 -> 3399 bytes
       .../src-tauri/icons/ios/AppIcon-20x20@3x.png       |  Bin 0 -> 6501 bytes
       .../src-tauri/icons/ios/AppIcon-29x29@1x.png       |  Bin 0 -> 2032 bytes
       .../src-tauri/icons/ios/AppIcon-29x29@2x-1.png     |  Bin 0 -> 6122 bytes
       .../src-tauri/icons/ios/AppIcon-29x29@2x.png       |  Bin 0 -> 6122 bytes
       .../src-tauri/icons/ios/AppIcon-29x29@3x.png       |  Bin 0 -> 12032 bytes
       .../src-tauri/icons/ios/AppIcon-40x40@1x.png       |  Bin 0 -> 3399 bytes
       .../src-tauri/icons/ios/AppIcon-40x40@2x-1.png     |  Bin 0 -> 10446 bytes
       .../src-tauri/icons/ios/AppIcon-40x40@2x.png       |  Bin 0 -> 10446 bytes
       .../src-tauri/icons/ios/AppIcon-40x40@3x.png       |  Bin 0 -> 20471 bytes
       .../src-tauri/icons/ios/AppIcon-512@2x.png         |  Bin 0 -> 1533005 bytes
       .../src-tauri/icons/ios/AppIcon-60x60@2x.png       |  Bin 0 -> 20471 bytes
       .../src-tauri/icons/ios/AppIcon-60x60@3x.png       |  Bin 0 -> 40994 bytes
       .../src-tauri/icons/ios/AppIcon-76x76@1x.png       |  Bin 0 -> 9631 bytes
       .../src-tauri/icons/ios/AppIcon-76x76@2x.png       |  Bin 0 -> 30439 bytes
       .../src-tauri/icons/ios/AppIcon-83.5x83.5@2x.png   |  Bin 0 -> 35826 bytes
       apps/mac-cleaner-desktop/src-tauri/src/commands.rs |   14 +
       .../src-tauri/src/commands/cleanup.rs              |  110 +
       .../src-tauri/src/commands/helpers.rs              |   39 +
       .../src-tauri/src/commands/quarantine.rs           |   60 +
       .../src-tauri/src/commands/startup.rs              |   79 +
       .../src-tauri/src/commands/system.rs               |   64 +
       .../src-tauri/src/commands/tool_cleanup.rs         |  131 +
       .../mac-cleaner-desktop/src-tauri/src/lifecycle.rs |    9 +
       apps/mac-cleaner-desktop/src-tauri/src/main.rs     |   89 +
       apps/mac-cleaner-desktop/src-tauri/src/monitor.rs  |   70 +
       apps/mac-cleaner-desktop/src-tauri/src/tray.rs     |  287 +
       apps/mac-cleaner-desktop/src-tauri/tauri.conf.json |   43 +
       apps/mac-cleaner-desktop/ui/index.html             |   13 +
       apps/mac-cleaner-desktop/ui/package-lock.json      | 1878 +++++++
       apps/mac-cleaner-desktop/ui/package.json           |   29 +
       apps/mac-cleaner-desktop/ui/public/app-icon.png    |  Bin 0 -> 22806 bytes
       apps/mac-cleaner-desktop/ui/src/App.tsx            |   67 +
       apps/mac-cleaner-desktop/ui/src/StorageSection.tsx |   97 +
       apps/mac-cleaner-desktop/ui/src/api.ts             |   41 +
       .../ui/src/components/ActivitySection.tsx          |   39 +
       .../ui/src/components/AppHeader.tsx                |   25 +
       .../ui/src/components/CleanupWorkspace.tsx         |   78 +
       .../ui/src/components/PlanReview.tsx               |   22 +
       .../ui/src/components/ProcessMonitor.tsx           |   55 +
       .../ui/src/components/QuarantineSection.tsx        |   37 +
       .../ui/src/components/SystemOverview.tsx           |   76 +
       apps/mac-cleaner-desktop/ui/src/format.test.ts     |   37 +
       apps/mac-cleaner-desktop/ui/src/format.ts          |   44 +
       .../ui/src/hooks/useMacCleaner.ts                  |  349 ++
       apps/mac-cleaner-desktop/ui/src/main.tsx           |   14 +
       apps/mac-cleaner-desktop/ui/src/styles/base.css    |   65 +
       .../ui/src/styles/components.css                   |  260 +
       apps/mac-cleaner-desktop/ui/src/types.ts           |  148 +
       apps/mac-cleaner-desktop/ui/src/vite-env.d.ts      |    1 +
       apps/mac-cleaner-desktop/ui/tsconfig.json          |   21 +
       apps/mac-cleaner-desktop/ui/tsconfig.node.json     |    9 +
       apps/mac-cleaner-desktop/ui/vite.config.ts         |   12 +
       crates/cleaner-domain/Cargo.toml                   |   15 +
       crates/cleaner-domain/src/catalog.rs               |   84 +
       crates/cleaner-domain/src/cleanup.rs               |  114 +
       crates/cleaner-domain/src/error.rs                 |   11 +
       crates/cleaner-domain/src/lib.rs                   |   17 +
       crates/cleaner-domain/src/quarantine.rs            |   95 +
       crates/cleaner-domain/src/storage.rs               |   43 +
       crates/cleaner-domain/src/system.rs                |   55 +
       crates/cleaner-domain/src/tool_cleanup.rs          |   71 +
       crates/cleaner-fs/Cargo.toml                       |   26 +
       crates/cleaner-fs/src/error.rs                     |   45 +
       crates/cleaner-fs/src/executor.rs                  |  192 +
       crates/cleaner-fs/src/lib.rs                       |   20 +
       crates/cleaner-fs/src/metadata.rs                  |  156 +
       crates/cleaner-fs/src/quarantine.rs                |  138 +
       crates/cleaner-fs/src/scanner.rs                   |  201 +
       crates/cleaner-fs/src/storage_analysis.rs          |  199 +
       crates/cleaner-fs/src/storage_catalog.rs           |  230 +
       crates/cleaner-fs/tests/safety.rs                  |  130 +
       crates/cleaner-fs/tests/storage_analysis.rs        |   88 +
       crates/cleaner-policy/Cargo.toml                   |   19 +
       crates/cleaner-policy/src/catalog.rs               |   85 +
       crates/cleaner-policy/src/error.rs                 |   35 +
       crates/cleaner-policy/src/lib.rs                   |   16 +
       crates/cleaner-policy/src/paths.rs                 |   65 +
       crates/cleaner-policy/src/plans.rs                 |  106 +
       crates/cleaner-policy/src/quarantine.rs            |   50 +
       crates/cleaner-policy/src/tests.rs                 |   37 +
       crates/cleaner-service/Cargo.toml                  |   28 +
       crates/cleaner-service/src/error.rs                |   63 +
       crates/cleaner-service/src/fs_safety.rs            |  125 +
       crates/cleaner-service/src/lib.rs                  |  288 +
       crates/cleaner-service/src/tool_cleanup/mod.rs     |  221 +
       .../src/tool_cleanup/simulator/discovery.rs        |  272 +
       .../src/tool_cleanup/simulator/mod.rs              |  214 +
       crates/cleaner-service/src/tool_cleanup/uv.rs      |  118 +
       crates/cleaner-service/tests/service_flow.rs       |  177 +
       crates/cleaner-store/Cargo.toml                    |   23 +
       crates/cleaner-store/src/cleanup_plans.rs          |  184 +
       crates/cleaner-store/src/error.rs                  |   57 +
       crates/cleaner-store/src/lib.rs                    |   65 +
       crates/cleaner-store/src/migration.rs              |  228 +
       crates/cleaner-store/src/quarantine.rs             |  218 +
       crates/cleaner-store/src/recovery.rs               |   54 +
       crates/cleaner-store/src/schema.rs                 |   93 +
       crates/cleaner-store/src/settings.rs               |   34 +
       crates/cleaner-store/src/state.rs                  |   83 +
       crates/cleaner-store/src/tool_cleanup.rs           |  261 +
       crates/cleaner-store/tests/migration.rs            |   51 +
       crates/cleaner-store/tests/store.rs                |  143 +
       crates/cleaner-system-macos/Cargo.toml             |   16 +
       crates/cleaner-system-macos/src/lib.rs             |  288 +
       rust-toolchain.toml                                |    5 +
       scripts/build-universal.sh                         |   23 +
       scripts/check-rust.sh                              |   15 +
       scripts/dev-rust.sh                                |   12 +
       179 files changed, 22135 insertions(+), 110 deletions(-)

2026-11-16 — Registered the native SwiftUI shell migration after the owner changed the target architecture.

2026-11-16 — Existing Rust CLI already emits JSON for system status, catalog scans, history, and quarantine lists; mutating flows already use opaque IDs and backend revalidation, but need a noninteractive app protocol.

2026-11-16 — Existing Tauri shell owns menu bar, 30-second monitoring, notifications, login startup, native confirmations, and packaging. These map directly to SwiftUI `MenuBarExtra`, AppKit activation, UserNotifications, ServiceManagement, and Xcode build phases.

2026-11-16 — Apple documents `MenuBarExtra` as the native persistent utility surface, `Bundle.url(forAuxiliaryExecutable:)` for locating bundled executables, standard helper placement under `Contents/MacOS` or `Contents/Helpers`, and `SMAppService` for user-controlled login/background registration.

2026-11-16 — Xcode 26.4.1 and Swift 6.3.1 are installed. No existing Swift target or generator constrains the new native project.

2026-11-16 — Owner approved the native SwiftUI migration plan; implementation started.

2026-11-16 — Added `mac-cleaner-engine`, a versioned one-shot JSON helper exposing only fixed commands and opaque backend IDs. No command accepts arbitrary cleanup paths or shell text.

2026-11-16 — Added the native Xcode project and SwiftUI/AppKit views for overview, storage attribution/history, cleanup review, quarantine, activity, settings, menu bar, notifications, and login-item preference.

2026-11-16 — Replaced the basic menu with a native window-style popover: compact RAM/SSD metric cards, accessible progress indicators, semantic status, and clear primary actions. The menu-bar label now reports available RAM and free SSD without alarm-only red text.

2026-11-16 — Replaced the constrained SwiftUI `MenuBarExtra` label with a native AppKit status item so RAM, CPU, and SSD each have a distinct pressure-colored SF Symbol and compact value. The SwiftUI popover remains the interactive surface. Installed universal-build QA confirmed red critical RAM, orange warning CPU, and green healthy SSD rendering.

2026-11-16 — Fixed the toolbar refresh state as one stable circular control: the centered spinner replaces the arrow in place, and the redundant outline that produced a doubled ring was removed.

2026-11-16 — Replaced the popover's generic sparkle tile with a project-bundled transparent Mac Cleaner mark generated for this product: a coral storage volume, mint health pulse, and safe cleanup sweep. Increased popover legibility with an opaque native surface, stronger secondary text, pressure-colored metric values/process percentages, and custom high-contrast progress bars.

## Release follow-up

- Apple requires App Sandbox for Mac App Store distribution. The current cleaner needs a deliberate consent/bookmark and filesystem-scope design before sandboxing can be enabled without breaking storage analysis.
- Configure Apple Development/Distribution identities, hardened runtime, App Store entitlements, archive upload, review metadata, and store QA in a separate release stream.

---

## 🔍 Audit Report

_Status: implementation and local verification complete; awaiting owner closure/commit approval._
