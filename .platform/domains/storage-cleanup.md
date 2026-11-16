---
domain_id: dom-storage-cleanup
slug: storage-cleanup
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [safe-cleanup-policy, storage-forensics, rust-application, native-macos-shell]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# storage-cleanup

Finds exact reviewable artifacts from a compiled Rust catalog and moves confirmed items to recoverable quarantine. Read-only storage attribution is handled separately by `storage-forensics` and never grants cleanup authority.

## Key files

- `crates/cleaner-policy/src/catalog.rs` — compiled cleanup roots and permanent protections.
- `crates/cleaner-fs/src/scanner.rs` and `executor.rs` — bounded discovery and identity-checked quarantine moves.
- `crates/cleaner-service/src/lib.rs` — plan preparation, approval, mutation locks, and receipts.
- `apps/mac-cleaner-macos/Sources/CleanupView.swift` — selection, review, and native confirmation.
- `apps/mac-cleaner-cli/` and `apps/mac-cleaner-engine/` — typed clients over the same service.

## Locked contracts

- Only fresh candidates rediscovered from the compiled catalog may be planned.
- System Data parent paths are report-only.
- Sizes use allocated blocks so sparse VM/container files are not overcounted.
- Never default-delete user documents, source repos, credentials, app support databases, or unknown data.
- Dart/Flutter's global Pub package store (`~/.pub-cache`) is report-only/protected; use Pub/Flutter commands for maintenance instead of deleting it from Mac Cleaner.
- Quarantine, restore, and purge are separate journaled actions; moving to quarantine does not claim reclaimed disk space.
