---
domain_id: dom-background-ai-optimizer
slug: background-ai-optimizer
status: retired
repo_ids: [mac-cleaner]
related_domain_slugs: [ai-assistant, memory-optimization, storage-cleanup, local-dashboard]
created_at: 2026-04-29
updated_at: 2026-11-16
---

# background-ai-optimizer

<!-- updated: 2026-11-16 — retired; the Python server, model adapter, and automatic purge behavior were removed. -->

## What this domain does

Historically added an AI decision layer over RAM, processes, and cleanup findings. The current product has no AI optimizer and never runs automatic RAM purge or model-selected actions.

## Backend / source of truth

- `crates/cleaner-policy/` and `crates/cleaner-service/` own deterministic safety and execution.
- `crates/cleaner-system-macos/` reports pressure without manufacturing free RAM.

## Frontend / clients

- The native app surfaces measurements and user-controlled actions directly; no recommendation model is present.

## API contract locked

- No model or background loop may approve, select, or execute cleanup.
- Protected developer processes stay protected, and memory recovery remains advisory.
- Storage cleanup uses Rust-owned IDs and plans; native requests cannot provide delete paths.

## Key files

- `crates/cleaner-policy/src/`
- `crates/cleaner-service/src/`
- `crates/cleaner-system-macos/src/lib.rs`

## Decisions locked

- Background AI is retired, not merely disabled.
- Any future AI integration requires a new reviewed domain and cannot bypass local policy or confirmation.
