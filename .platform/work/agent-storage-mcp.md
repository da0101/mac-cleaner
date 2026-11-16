---
stream_id: stream-agent-storage-mcp
slug: agent-storage-mcp
type: feature
status: in-progress
agent_owner: codex
domain_slugs: [storage-forensics, rust-application, safe-cleanup-policy, native-macos-shell]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: feature/full-rust-rewrite
created_at: 2026-11-16
updated_at: 2026-11-16
closure_approved: false
---

# agent-storage-mcp

## Scope

- Bundle a local Rust MCP stdio server with the native application.
- Let Codex and Claude inspect the same path-level storage reports, history, system health, and conservative discovery output as the app.
- Let agents use bounded Rust-owned cleanup plans while preserving every existing authorization boundary.
- Provide discoverable setup in the native Settings UI and documented registration commands.

## Done criteria

- [x] Installed app contains a universal `mac-cleaner-mcp` helper.
- [x] Codex and Claude both connect and list the intended tools.
- [x] Tool results contain structured exact-path evidence and useful compact text.
- [x] Inspection tools are read-only; prepare tools are non-destructive; execution tools are explicitly destructive and closed-world.
- [x] No arbitrary path is exposed; mutations require opaque IDs, a five-minute exact plan, explicit user approval, and a plan-specific confirmation.
- [x] Rust, Swift, packaging, and live MCP smoke tests pass.
- [ ] Owner completes visual and agent-access QA before stream closure.

## Key decisions

2026-11-16 — Use the official Rust MCP SDK over local stdio and bundle the helper inside the `.app`; this preserves the Swift/Rust-only product, avoids a localhost listener, and lets both agent hosts launch the same reviewed binary.

2026-11-16 — Superseded: agents may now prepare and execute only Rust-owned exact plans. General items move to Recovery first; permanent purge is a separate confirmed action; fixed tool cleanup covers only uv cache and verified old simulator recordings.

2026-11-16 — Include the snapshot UUID and total hotspot count in compact text as well as structured content because agent hosts may summarize text first; exact-path structured output remains authoritative.

## Resume state

- **Last updated:** 2026-11-16 by codex
- **What just happened:** Verified the installed helper exposes all 11 inspection, prepare, and exact-confirmation execution tools; live `system_health` returned CPU/RAM/SSD evidence and ranked processes.
- **Current focus:** Owner visual and interaction QA; no real cleanup execution occurred during MCP QA.
- **Next action:** Owner reviews the installed menu-bar status and starts a fresh Codex/Claude session when ready to use the expanded tool inventory.
- **Blockers:** none

## Progress log

2026-11-16 — Owner requested direct Codex and Claude access to Mac Cleaner's storage evidence. Registered this feature stream using the documented fallback because the machine's `ab` command resolves to ApacheBench rather than Agentboard.

2026-11-16 — Added four read-only tools (`latest_storage_report`, `analyze_storage`, `system_health`, and `cleanup_candidates`) backed by the same Rust service and SQLite history as the app. No arbitrary path or mutation tool exists.

2026-11-16 — Universal packaging and installed-app QA passed. Live Codex and Claude calls both returned `hotspots=25` and snapshot `9a425b80-54d7-44df-8140-73b0afdd7edc`.

2026-11-16 — Added a first-class Agents tab showing helper readiness, the four inspection tools, setup commands, a safe review prompt, and the explicit boundary that agents diagnose while Mac Cleaner authorizes cleanup.

2026-11-16 — Owner explicitly requested Claude/Codex cleanup authority. Added prepare/execute pairs for recoverable catalog cleanup, fixed tool-owned cleanup, and single-item Recovery actions. Execution tools are MCP-destructive, reject paths, require a plan-bound phrase, revalidate through the same Rust service, and retain repository/toolchain/database protections.

2026-11-16 — Added CPU usage/pressure and a union of top-memory/top-CPU processes to `system_health`, including an actual sampling interval so short-lived one-shot helpers report useful per-process CPU values.

2026-11-16 — Installed-helper JSON-RPC smoke QA passed for initialization, all 11 tool definitions/annotations, and read-only `system_health`. `codex mcp list` reports the installed helper enabled, and `claude mcp list` reports it connected. No prepare or destructive tool was invoked.
