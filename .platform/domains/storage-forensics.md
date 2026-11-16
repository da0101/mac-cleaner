---
domain_id: dom-storage-forensics
slug: storage-forensics
status: active
repo_ids: [mac-cleaner]
related_domain_slugs: [rust-application, storage-cleanup, safe-cleanup-policy, native-macos-shell]
created_at: 2026-11-16
updated_at: 2026-11-16
---

# storage-forensics

Read-only storage attribution and historical growth tracking for the Rust desktop app.

## Intended cross-layer touch points

- `crates/cleaner-domain/src/storage.rs` — typed findings, categories, risk, and snapshot/delta contracts.
- `crates/cleaner-fs/src/storage_analysis.rs` and `storage_catalog.rs` — allocated-size discovery for known storage consumers.
- `crates/cleaner-service/` — bounded analysis orchestration and snapshot comparison.
- `crates/cleaner-store/` — local SQLite snapshot persistence and retention.
- `apps/mac-cleaner-engine/src/main.rs` — fixed typed read-only command exposure.
- `apps/mac-cleaner-mcp/` — bundled stdio MCP exposure of the same saved reports and live health data for Codex and Claude.
- `apps/mac-cleaner-macos/Sources/StorageView.swift` and supporting native models — compact filterable table, path details, and copy/export controls.
- `apps/mac-cleaner-cli/src/commands/reporting.rs` — machine-readable/plain-text report parity where appropriate.

## Locked contracts

- Measurement is read-only and uses allocated disk usage where available.
- Measurement does not cross filesystem or mounted-runtime boundaries unless a mount is explicitly catalogued as its own scan root.
- APFS clones and shared blocks cannot always be attributed perfectly; volume change and measured hotspot change remain separate values with an explicit accounting gap.
- Findings never become deletion authority; cleanup remains plan/confirmation/revalidation gated.
- Repositories, databases, SDKs, active toolchains, credentials, user documents, and unknown paths remain protected even when large.
- Every finding exposes the exact path, category, safety classification, measurement time, and confidence/source.
- Historical comparisons use locally persisted snapshots and make scan coverage changes explicit so they are not misreported as growth.
- Copy/export output contains paths and sizes but no file contents, credentials, or environment values.
- Agent access is closed-world: MCP accepts no arbitrary path and exposes no process termination, Docker prune, SDK/runtime deletion, repository mutation, or unknown-data cleanup. Mutations require opaque Rust candidate IDs, a short-lived exact plan, explicit confirmation, revalidation, and a separate destructive call; simulator mutation is limited to verified old test recordings.
