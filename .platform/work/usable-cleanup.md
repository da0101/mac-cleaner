# usable-cleanup

Type: feature · Status: in-progress · Agent: claude · Started: 2026-11-16

## Why
Owner QA: app analyses storage but cannot free anything. Safe scan always returned 0
(Flutter DerivedData rejected by deep sentinel match; node_modules/target rejected for
internal symlinks/hard links; Documents wholly protected). Every other action was Reveal in Finder.

## Approved scope (owner "ok", 2026-11-16)
1. Real per-row cleanup through the existing plan → confirm → revalidate → Recovery flow.
2. Fix eligibility rules (sentinel depth, generated-tree links, explain too-recent skips).
3. One "Free up space" checklist with a single Clean selected action; Empty Recovery.
7. Window reopen after close.
Deferred: RAM/CPU quit actions (needs owner decision vs hard constraint #3), Docker breakdown, FDA onboarding.

## Design
- cleaner-policy `TargetLayout`: Children (existing) | Named(entry) | Project{marker, entry}.
- Project targets live under Documents/GitHub/<p>[/<sub>]/<entry>; narrow exception to the
  Documents protection, only for fixed generated names next to a marker file.
- Candidates are whole directories moved by rename into Quarantine; purge is separate.

## Knowledge delta
- `open -a` reopen failed because bringMainWindowForward only finds existing windows.

## Checkpoint 2026-11-16 15:10
Done: Rust layout-aware catalog + eligibility fixes (8 new tests; clippy and workspace tests green).
Real read-only scan: 14.24 GB / 11 candidates (was 0). New Free Up Space screen, Empty Recovery,
reopen fix. Universal build installed to ~/Applications. Nothing deleted.
Next: restart Terminal so Accessibility/Screen Recording apply → click-through QA.
Open: `xcodebuild test` reports "Could not launch MacCleanerNativeTests" — check if pre-existing.
Do not commit without owner approval.

## Checkpoint 2026-11-16 (later)
- Found via engine dry runs: every engine invocation ran fail_incomplete_on_startup, so the app's
  5 s status poll marked in-flight plans failed → cleans >~5 s (node_modules) errored after moving.
  Fixed: recovery runs only when mutation.lock is free (cleaner-service `try_mutation_lock`). Built, not installed
  (installing was blocked by the permission check in this session).
- Click-through handed off to Claude desktop: `QA_CLICKTHROUGH_HANDOFF.md`.

## Checkpoint 2026-11-16 (cross-computer handoff)
- Owner explicitly requested commit + push of the complete working tree to continue elsewhere.
- Full click-through remains open. Codex could not drive the UI because macOS denied Assistive
  Access; the newest engine build with the mutation-lock fix is not yet visually verified.
- MCP diagnosis during critical pressure: 14.79 GB volume growth from the 12:56 PM baseline,
  including 12.82 GB of system swap/accounting gap and 1.96 GB measured build/cache growth.
  `/System/Volumes/VM` was 24 GB; Docker grew only 6.5 MB and GreyBox data remained untouched.
- No cleanup was executed. Active Mac Cleaner, TakeCare, Vibe Bloom, Xcode/mobile tools,
  repositories, and Docker/GreyBox data were retained.
- Continue with `QA_CLICKTHROUGH_HANDOFF.md` from the pushed `feature/full-rust-rewrite` branch.
