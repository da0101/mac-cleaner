# mac-cleaner — Current Status

Last updated: 2026-11-16

> Local-only macOS storage forensics, conservative cleanup, quarantine recovery, and memory-pressure visibility for developer machines.

## Feature areas

| Area | Status | Last touched | Notes |
|---|---|---|---|
| Native macOS app | 🟢 Live candidate | 2026-11-16 | Pure SwiftUI/AppKit app with overview, storage, cleanup, quarantine, activity, settings, notifications, and a modern window-style menu-bar popover. |
| Rust app engine | 🟢 Live candidate | 2026-11-16 | Bundled one-shot helper exposes protocol v1 fixed commands and opaque IDs; it accepts no arbitrary cleanup path or shell text. |
| CLI storage scanner | 🟢 Live candidate | 2026-11-16 | `apps/mac-cleaner-cli/` provides read-only scans plus typed confirmation flows over the shared Rust services. The former `cleaner.py` implementation is retired. <!-- stale: Python CLI removed during native rewrite --> |
| Shared scanner and policy | 🟢 Live candidate | 2026-11-16 | `cleaner-fs` and `cleaner-policy` use allocated size, compiled catalogs, repository/toolchain protections, identity revalidation, and fail-closed execution. The former `scanner.py` is retired. <!-- stale: Python scanner removed during native rewrite --> |
| Storage forensics/history | 🟢 Live candidate | 2026-11-16 | Read-only known-path attribution, manifest-aware generated output, SQLite snapshots/baselines, deltas, and accounting-gap reporting are available in the native app. |
| Cleanup and quarantine | 🟢 Live candidate | 2026-11-16 | Exact candidates become short-lived plans, require native confirmation, move to recoverable quarantine, and support separately confirmed restore/purge. |
| RAM/process monitor | 🟢 Live candidate | 2026-11-16 | Reports conservative available/reclaimable/compressed/swap metrics, protects developer jobs, notifies on sustained pressure, and never kills apps automatically. |
| Universal packaging | 🟢 Verified locally | 2026-11-16 | App, Rust helper, and CLI build for arm64+x86_64; unsigned development DMG passes local packaging. Store signing/sandboxing remains open. |
| Local dashboard | ⚪ Retired | 2026-11-16 | The localhost Python/HTML dashboard was removed and replaced by the native app. <!-- stale: v1 dashboard no longer exists on this branch --> |
| Background AI optimizer | ⚪ Retired | 2026-11-16 | AI-driven background cleanup was removed; local Rust policy and explicit user confirmation are authoritative. <!-- stale: v1 AI optimizer removed --> |
| Browser tab optimizer | ⚪ Retired | 2026-11-16 | Chrome tab inspection/closing was removed from the product scope. <!-- stale: v1 browser automation removed --> |
| AI assistant | ⚪ Retired | 2026-11-16 | The legacy terminal assistant and provider dependencies were removed. <!-- stale: v1 Python assistant removed --> |
| Open-source packaging | 🟡 Release follow-up | 2026-11-16 | License/CODEOWNERS remain; Developer ID/App Store signing, sandboxed filesystem consent, notarization where applicable, and submission QA are outstanding. |

## Immediate priorities

1. **Owner review and commit** — review the native app and menu-bar UX, then approve the migration commit.
2. **Distribution design** — choose direct Developer ID distribution or design sandbox-safe filesystem access for Mac App Store submission.
3. **Installed-app cutover** — replace any legacy login item only through explicit user-controlled installation/startup migration.

## Known gotchas

- Developer tools, terminal sessions, agents, compilers, and active jobs must never be killed automatically.
- Storage findings are explanatory only; they do not become deletion authority without independent catalog discovery and a fresh plan.
- Sparse VM/container files use allocated disk usage, and APFS shared blocks can leave an explicit accounting gap.
- `~/.pub-cache`, Android/Xcode SDKs, simulator runtimes, repositories, Docker databases, credentials, and unknown paths remain protected.
- Mac App Store distribution requires App Sandbox; the current broad read-only analysis model needs an explicit user-consent/bookmark design first.
