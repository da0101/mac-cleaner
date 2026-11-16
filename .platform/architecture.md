# mac-cleaner — Architecture

Last updated: 2026-11-16

## What this system does

mac-cleaner is a local-only macOS cleanup tool. It finds safe-to-delete caches/logs/dev artifacts, reports large System Data contributors, shows RAM availability, and surfaces heavy processes without interrupting active developer work.

## Components

```text
apps/mac-cleaner-macos/  -> native SwiftUI/AppKit app and menu bar
apps/mac-cleaner-engine/ -> fixed-command JSON bridge into Rust
apps/mac-cleaner-cli/    -> terminal interface using the same Rust services
crates/cleaner-domain/   -> typed contracts and opaque identifiers
crates/cleaner-policy/   -> compiled safety allowlist and protections
crates/cleaner-fs/       -> bounded discovery and identity-checked actions
crates/cleaner-service/  -> plans, confirmations, orchestration, receipts
crates/cleaner-store/    -> SQLite history, snapshots, and quarantine journal
crates/cleaner-system-macos/ -> read-only disk, memory, and process snapshots
```

## Data flow

```text
SwiftUI action -> fixed EngineCommand -> bundled mac-cleaner-engine
              -> AppService -> policy + filesystem + SQLite
              <- protocol-v1 JSON <- typed Swift models <- native view
```

The app opens no listener. Read-only status refreshes every 30 seconds; storage analysis is on demand. Mutations require a prepared Rust plan, a native confirmation, backend approval, identity revalidation, and a receipt.

## Stack

| Layer | Choice |
|---|---|
| Native UI | SwiftUI, AppKit, ServiceManagement, UserNotifications |
| Backend | Rust 2024 workspace |
| App boundary | Bundled one-shot helper, versioned JSON, fixed commands |
| State | SQLite WAL plus typed Rust domain models |
| Distribution | Universal native `.app`, CLI, and DMG |

## Invariants

1. Never delete user documents, photos, source repos, credentials, app databases, keychains, or unknown data by default.
2. Destructive cleanup must be preview/confirmation-first.
3. The app opens no network listener and Swift never passes arbitrary paths or shell fragments to Rust.
4. Developer tools, shells, agents, compilers, and active jobs are protected unless the user explicitly approves interruption.
5. System Data discovery is path-level and report-only unless a child path is classified safe.
6. Secrets are never committed, printed, or logged.
7. Memory recovery is advisory and pressure-based; the app never auto-terminates processes or manufactures free memory.
8. Shared developer package stores such as Dart/Flutter's `~/.pub-cache` are report-only unless maintained through their own package-manager commands.

## Known debt

| Area | Issue | Planned fix |
|---|---|---|
| Distribution signing | Local builds are unsigned development artifacts. | Complete Developer ID signing and notarization before release. |
| Mac App Store sandbox | Broad developer-storage analysis is not yet modeled through sandbox-scoped user consent. | Design security-scoped access/bookmark scopes before enabling App Sandbox. |
