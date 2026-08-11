---
stream_id: stream-full-rust-rewrite
slug: full-rust-rewrite
type: refactor
status: in-progress
agent_owner: codex
domain_slugs: [rust-application, safe-cleanup-policy]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: feature/full-rust-rewrite
created_at: 2026-08-10
updated_at: 2026-08-11
closure_approved: false
---

# full-rust-rewrite

## Scope

- Replace the shipped Python runtime, server, CLI, menu-bar app, scheduler, and installers with Rust.
- Preserve or deliberately retire every current user-facing capability through an explicit parity matrix.
- Make the safe-cleanup contract foundational instead of porting the current deletion policy.
- Deliver a dependable macOS login app, local UI, settings/history migration, universal packaging, and rollback.
- Keep Python only as a migration reference until the Rust product passes release gates.

## Approved architecture

- Tauri 2 desktop shell with bundled React/TypeScript UI assets and no shipped Node runtime.
- Shared Rust domain, policy, filesystem, system, store, and service crates.
- Separate Rust CLI linked to the same service and policy crates.
- No localhost HTTP listener; narrowly allowlisted Tauri commands and channels only.
- SQLite operation journal and settings/history store.
- Initial automatic storage-cleanup allowlist is empty.
- Generic product flow is fixed as analyze → explain → offer reviewed actions → exact preview → user confirmation → verified receipt.
- Large orchestration/UI/store files are split by responsibility; new capabilities belong in reusable modules rather than monolithic entry points.

## Approved retirements

- Generic Clean All and unattended storage deletion.
- Startup RAM purge, memory-pressure generation, process signaling, and app manipulation.
- Legacy OpenAI terminal executor, DNS flush, combined destructive shortcuts, and menu fallback deletion.
- Docker prune-with-volumes defaults and arbitrary port-owner termination.

## Done criteria

- [x] Shipped app and CLI run without Python or pip.
- [x] Adversarial fixture suite passes with zero forbidden-path deletion.
- [ ] Startup, menu bar, monitoring, scan/preview/clean, settings/history, and approved integrations meet parity.
- [ ] Arm64 and Intel/universal builds, upgrade, uninstall, crash recovery, signing, notarization, and rollback are verified.
- [x] Unit, integration, UI, security, and manual QA artifacts pass without real-user destructive tests.
- [ ] Python retirement is owner-approved only after a clean full-repo re-audit.
- [ ] `.platform/memory/log.md` and `decisions.md` document the outcome.

## Worktree / Local environment

| Repo | Worktree path | Branch | Base | Dependencies | Local command | Port |
|---|---|---|---|---|---|---|
| mac-cleaner | `/Users/danilulmashev/Documents/GitHub/mac-cleaner-full-rust-rewrite` | `feature/full-rust-rewrite` | `main` | Rust 1.96, Node 22 build-time, Xcode 26 | `./scripts/dev-rust.sh` | none shipped; dev Vite 1420 |

## Resume state

- **Last updated:** 2026-08-11 by codex
- **What just happened:** The owner confirmed an exact 453-file simulator-recording plan and Mac Cleaner permanently removed 22.5 GB of verified old QuickTime/XCTest recordings. The simulator, apps, app data, source code, and other test results remained outside the target. A portable-timestamp serialization regression was fixed, the operation now appears in unified Recent activity, and available storage rose to about 50.2 decimal GB.
- **Current focus:** Production hardening of the generic analyze → explain → reviewed action → confirmation → receipt flow.
- **Next action:** Finish startup/update/distribution parity, universal Intel QA, signing/notarization, and the explicit legacy Python cutover audit.
- **Blockers:** Developer ID/notarization/updater credentials and a physical/hosted Intel QA environment are not available in this workspace.

## Progress log

2026-08-10 — Full Rust rewrite plan approved and isolated feature worktree created from `main`.

2026-08-10 — Implemented compiled review-only catalog, permanent development/application-state protections, immutable plan approvals, TOCTOU revalidation, private quarantine, one-entry restore/purge, SQLite crash journal, system monitor, Rust CLI, legacy JSON migration, Tauri tray/lifecycle/startup gate, and local bundled UI.

2026-08-10 — Full gate passed: clippy warnings denied, 24 Rust tests, frontend tests/build, zero npm advisories, native read-only QA, universal CLI/app architectures, and valid DMG checksum. Added and packaged the production shield/sweep/sparkle logo.

2026-08-10 — Retired `com.mac-cleaner.menubar` after verifying its Python process and moved its plist to a dated application-support backup. Added Rust menu-bar RAM/SSD availability, critical-storage UI, and bounded read-only hotspot analysis with symlink/repository protection tests. Packaging verification created 3.1 GiB of temporary Cargo output; preserved the app bundle and removed only that generated output afterward.

2026-08-10 — Confirmed the running simulator app matches `vibe-bloom`, while the 10.3 GB `astral-flutter/app/build` artifact hash does not match and has been inactive since 2026-07-27. Prepared menu `available / total` formatting, process headers, stable columns, and collapsed hotspot results; packaging is paused pending owner-approved emergency cleanup.

2026-08-10 — Owner approved the exact inactive Astral target. Ran `flutter clean` only in `astral-flutter/app`, removed its generated `build/` and `.dart_tool/` output, verified a clean Git worktree plus preserved `pubspec.yaml`/`pubspec.lock`, and left the active Vibe simulator/build untouched. Reclaimed about 10.7 GB, packaged/reopened the revised Rust app, then removed only temporary Cargo build artifacts.

2026-08-10 — Corrected the Rust RAM metric after live comparison exposed that `sysinfo::available_memory` includes active pages and reported about 7.5 GB where the conservative reclaimable-page calculation reported about 3 GB. Rust now reads the fixed `/usr/bin/vm_stat`, discovers its page size, reports available plus completely-free RAM separately, and falls back safely if the command fails. Added relative per-process high/very-high labels and a compact native AppKit menu title with independently colored RAM/SSD text and no emoji.

2026-08-10 — Connected storage diagnosis to the reviewed cleanup flow with a prominent `Find safe cleanup` action and per-card owning-tool launch actions. Added a live rather than stale availability summary and an automatic 30-second low-memory watchdog that alerts below 1 GB without purging or closing apps. Live QA found no current generic-catalog candidates and one protected/unsafe skip; Docker is stopped, while 31.6 GB of 34.2 GB simulator data belongs to the booted iPhone and only about 2.2 GB belongs to a shutdown iPad. Next release gate is a journaled owning-tool action plan for `uv cache prune` and individually selected shutdown simulators—never raw deletion of Docker's disk or active simulator data.

2026-08-10 — Implemented app-native uv pruning with an immutable expiring plan, fixed trusted Homebrew executable and arguments, native exact-path confirmation, cross-process mutation lock, root identity revalidation, SQLite state/events/receipt journal, and no `--force`. Live owner-confirmed execution reclaimed 516,743,168 allocated bytes; a second click found nothing else. Fixed the misleading primary discovery button, disabled duplicate execution, added visible/persistent receipts, packaged the app, and removed 3.7 GiB of temporary Cargo output with `cargo clean` after copying the bundle.

2026-08-11 — Owner clarified the generic product contract: Mac Cleaner analyzes RAM/storage, explains recognized consumers, offers concrete reviewed actions, executes only after explicit confirmation, and reports verified recovery. Began the missing simulator-test-recording action with 72-hour age gating, QuickTime signature checks, exact file identities, test-job blocking, one-device shutdown confirmation, and SQLite receipts. Dispatched parallel modularization of the React app, desktop commands, and store; the 1,200-line service is the next split target.

2026-08-11 — Completed the modularity pass: service `lib.rs` fell from 1,200 to 288 lines, store from 543 to 65, domain from 432 to 17, policy from 361 to 16, CLI `main.rs` from 352 to 21, desktop command facade from 430 to 14, and React `App.tsx` from 489 to 67. Cohesive modules preserve the public APIs and safety boundaries.

2026-08-11 — Fixed the 453-item simulator plan failure (`i128 is not supported`) by storing portable nanosecond timestamps as checked `i64` values and added a 500-item SQLite round-trip regression. Owner-confirmed execution then removed 453 verified recordings and reclaimed 22.5 GB. The app now releases its busy state before background re-analysis, removes the stale action immediately, merges tool receipts into Recent activity, attaches native confirmation to the main window, and uses a denser severity-coded UI. Full workspace tests, warnings-denied Clippy, frontend tests/build, debug app packaging, and installed-app visual QA pass.

## Complexity and risk

- Complexity: very high; full product and distribution replacement.
- Risk: high because the application mutates filesystem state.
- Mitigation: empty automatic allowlist, fixture-only destructive tests, immutable plans, native confirmation, fail-closed revalidation, cross-process locking, staged cutover, and retained Python rollback reference.

## Audit report

Status: core implementation and local audit pass; production signing, notarization, Intel device QA, and explicit legacy startup cutover remain open release gates.
