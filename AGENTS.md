<!-- agentboard:root-entry:begin v=1 -->
# mac-cleaner

**What this is:** A local-only native macOS app for explaining storage growth, reviewing conservative cleanup plans, recovering quarantined items, and monitoring memory pressure without interrupting developer work.

## Stack

Swift 6 with SwiftUI/AppKit for the macOS interface; a Rust 2024 workspace (Rust 1.96) for policy, filesystem safety, system metrics, SQLite persistence, CLI, and the bundled protocol-v1 helper. No Python, TypeScript, JavaScript app code, Node UI tooling, web view, React, or Tauri ships on this branch.

## Repo Structure

Single GitHub repository: `da0101/mac-cleaner`.

## How This Project Actually Works

- `apps/mac-cleaner-macos/` is the native Xcode project, SwiftUI views, menu bar, settings, notifications, and login-item preference.
- `apps/mac-cleaner-engine/` is a bundled one-shot Rust executable with fixed commands, versioned JSON, and opaque identifiers; it accepts no arbitrary cleanup path or shell text.
- `apps/mac-cleaner-cli/` exposes the same Rust service for terminal workflows.
- `crates/` separates domain contracts, policy, filesystem discovery/actions, orchestration, SQLite state, and read-only macOS metrics.
- Every mutation starts with Rust discovery and a short-lived plan, requires native or typed confirmation, revalidates identity, and records a receipt.
- `./scripts/check-rust.sh`, `./scripts/dev-rust.sh`, and `./scripts/build-universal.sh` are the supported build/test/package entry points.

## Hard Constraints (Don't Break These)

1. Never delete user documents, photos, source repos, credentials, app databases, keychains, or unknown data by default.
2. Clients never supply cleanup paths. Rust discovers versioned candidates and owns plans, approvals, revalidation, quarantine, restore/purge, and receipts.
3. Never close, kill, signal, or manipulate editors, agents, terminals, compilers, or active developer jobs automatically. Exception, owner-approved 2026-11-17: apps the owner explicitly lists in Auto-Clean (never developer tools, Docker, or system apps) may be asked to quit the normal way when memory stays low.
4. Memory recovery is pressure-based and advisory: never invoke `purge`, manufacture memory pressure, or claim disk-cache flushing frees application RAM. Auto-Clean only asks opted-in apps to quit; it still never runs `purge` or kills a process.
5. Open no localhost server or web view; Swift invokes only the fixed helper resolved from the application bundle.
6. Do not read, print, or log `.env` contents, credentials, file contents, or other secrets.
7. Storage findings are explanatory only and never become deletion authority without independent catalog discovery and a fresh confirmed plan. Exception, owner-approved 2026-11-17: Auto-Clean may remove only catalog categories the owner opted into, idle 24+ hours, through the same plan, quarantine, purge, and receipt pipeline.
8. Keep Xcode/Android SDKs and runtimes, repositories/worktrees, package stores, Docker databases, credentials, and unknown paths protected.
9. Use SwiftUI/AppKit for presentation and Rust for backend behavior; do not reintroduce Python or TypeScript product code.

## Carried-over rules

_Kept from sections the kernel replaced (adopted 2026-11-16). Delete any that no longer apply._

- Follow `.platform/workflow.md`: triage, interview, research, propose, execute, verify. New non-trivial work must create a tracked stream in `.platform/work/ACTIVE.md` before implementation.


<!-- agentboard:kernel:begin v=1 -->
## Agentboard

Shared work-state for Claude Code, Codex and Gemini. Long procedures are on demand — `ab protocol` lists them, `ab protocol <name>` prints one.

**Session start:** run `ab brief --start`. One open stream → resume it (`ab handoff <slug>`); several → ask which; none → ask what to work on.

**Loading:** open only files named in `work/BRIEF.md` § "Relevant context". For a feature or topic run `ab recall <topic> [--layer backend|web|mobile]` before opening a domain file. Never read `work/archive/*`.

**No `ab` command on this machine?** Skip every `ab …` step: read `.platform/CARDS.md`, open only the one card that matches your task, and leave `work/archive/*` alone.

**Hard rules**
1. Task not in `work/ACTIVE.md`? Run `ab protocol new-task` before any research or code. New stream: research first, present the plan, wait for human approval.
2. Work on a `feature/<slug>` or `bugfix/<slug>` branch in the main checkout (`hotfix/<slug>` only if told). Worktrees are on demand: `ab worktree new <slug>`.
3. Only the human closes a stream (`ab protocol closure`). Record durable facts in the stream's `## Knowledge delta` as you work.
4. Change needs human or app-driving verification? `ab protocol qa` before any commit, push, merge or release.
5. Ending, low on context, or switching provider: `ab checkpoint <slug> --what "…" --next "…"` (`ab protocol handoff`).
6. After an edit another agent must understand: `ab log-reason "<why>"` (`ab protocol reasoning`).
7. **Role activation:** on the first task message match its intent to a role in `.platform/roles/INDEX.md` and adopt it; no confident match → `pair-programmer`, silently (`ab protocol roles`).
8. Audit or analysis of a stream: `ab protocol audit`.
9. User says "update the platform", "rescan" or "you're out of date": run `ab rescan`, then follow `.platform/RESCAN.md`.

**Silicon Valley product mindset:** act like best-in-class Silicon Valley product teams — user-obsessed, craft-driven, fast, rigorous — never as permission for scope creep, weak tests or skipping approval gates (`.platform/workflow.md`).

**Claude Code:** run these rules before any skill, including `using-superpowers`. Pass `model` on every `Task` call (`ab route <type> --for claude` says which) and print a dispatch manifest first (`ab protocol subagents`). Hooks enforce the closure gate.
<!-- agentboard:kernel:end v=1 -->
<!-- agentboard:root-entry:end v=1 -->
