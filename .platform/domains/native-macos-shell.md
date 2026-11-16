---
domain_id: dom-native-macos-shell
slug: native-macos-shell
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [rust-application, storage-forensics, safe-cleanup-policy]
created_at: 2026-11-16
updated_at: 2026-11-16
---

# native-macos-shell

Native SwiftUI presentation and lifecycle for the macOS-only product, backed by the existing Rust engine.

## Intended cross-layer touch points

- `apps/mac-cleaner-macos/` — Apple-native SwiftUI/AppKit application, models, views, resources, and Xcode project.
- `apps/mac-cleaner-engine/` — constrained versioned JSON command surface used by the native app without accepting arbitrary filesystem paths.
- `apps/mac-cleaner-mcp/` — bundled read-only stdio bridge for Codex and Claude storage inspection.
- `crates/cleaner-domain/` — serialized contracts shared across the process boundary.
- `crates/cleaner-service/` and `crates/cleaner-store/` — authoritative orchestration, policy, history, and persistence.
- `scripts/build-native-engine.sh` and `scripts/build-universal.sh` — build and embed the Rust helper and package the native `.app` bundle.

## Locked contracts

- UI uses Apple frameworks only: SwiftUI, AppKit, Foundation, and other system frameworks. No web view, React, Tauri, or third-party UI package ships in the final app.
- Active product source is Swift and Rust only; Python, TypeScript, JavaScript application code, Node tooling, and web UI build artifacts are absent from this branch.
- Rust remains authoritative for filesystem discovery, storage measurement, safety policy, cleanup plans, execution, recovery, and SQLite state.
- Swift sends typed identifiers and bounded options, never arbitrary cleanup paths or shell fragments.
- The Rust helper is bundled inside the application and invoked by an absolute bundle path with a fixed argument allowlist.
- Native confirmation occurs after a Rust plan is prepared and before the Rust engine approves or executes it.
- Existing protected repositories, databases, SDKs, runtimes, credentials, and unknown paths remain protected across the new boundary.
- The menu-bar title shows color-coded live RAM and SSD headroom with text labels so severity remains legible without relying on color alone.
- The app is a regular foreground macOS application, not `LSUIElement`; when focused it owns the macOS menu while retaining its `MenuBarExtra` status surface.
- Startup reloads persisted receipts and recovery entries, then runs read-only storage analysis and the fixed cleanup-catalog scan by default. The preference and last selected workflow tab persist in UserDefaults.
- Empty states remain actionable: Cleanup exposes owning-tool actions, Recovery links back to candidate discovery, History links to remeasurement, and Agents exposes MCP setup and review prompts.
