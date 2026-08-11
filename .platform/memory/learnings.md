# Platform Learnings — Bug Post-Mortems & Hard-Won Patterns

> **When to write here:** after fixing a non-obvious bug (one that took >10 min to diagnose OR required reading unfamiliar internals). Append immediately in Stage 6.
> **When to read here:** before diagnosing any non-obvious bug — grep this file first by symptom keyword. Don't re-diagnose a known class of problem.
> **Never auto-loaded** at session start. Load only when investigating a bug or appending a new entry.

Format:

```
## L-NNN — <short title>
Date: YYYY-MM-DD | Repo: <repo>
Symptom: <what the user/developer sees>
Root cause: <the actual reason>
Fix: <what was changed and where>
Class: <the category of problem — so you can grep it>
```

---

## L-004 — `npm cache verify --dry-run` Still Mutates the Cache
Date: 2026-08-10 | Repo: mac-cleaner
Symptom: A command intended as a read-only cache preview reports that content was garbage-collected.
Root cause: The installed npm does not implement `--dry-run` for `npm cache verify`; the unknown option does not prevent verify's normal garbage collection.
Fix: Never use `npm cache verify` during read-only analysis, with or without `--dry-run`. Treat it as a reviewed owning-tool cleanup action and disclose that it may remove cache content. The live audit removed 169.6 MB of npm cache garbage but did not touch projects, lockfiles, or installed dependencies.
Class: storage-cleanup npm dry-run owning-tool mutation

## L-003 — “Available Memory” Is Not One Universal macOS Number
Date: 2026-08-10 | Repo: mac-cleaner
Symptom: The Rust menu reports about 7.5 GB available while the legacy monitor reports about 3 GB available and roughly 70 MB raw free on the same Mac.
Root cause: `sysinfo::available_memory` on macOS includes active pages in its broader non-compressed estimate. The product had previously defined available as free, inactive, speculative, and purgeable pages, so the port silently changed semantics while keeping the same label.
Fix: Parse the fixed `/usr/bin/vm_stat` output in Rust, discover the page size from its header, preserve raw free separately, and calculate conservative available memory from free + inactive + speculative + purgeable pages. Fall back to bounded native library values if `vm_stat` fails, and keep raw free secondary in the UI.
Class: memory-monitoring metric-semantics macos vm-stat sysinfo

## L-002 — Bundle ID Alone Cannot Identify the Active Flutter Project
Date: 2026-08-10 | Repo: mac-cleaner
Symptom: A very large inactive Flutter build looks cleanable, but another running simulator app uses the same bundle ID, making activity ambiguous.
Root cause: Related project copies can produce apps with the same bundle ID, so process and bundle metadata do not identify which source tree produced the installed simulator binary.
Fix: Before offering project cleanup, compare a stable compiled artifact hash from the simulator-installed app with each candidate build. The installed Runner matched `vibe-bloom`, not `astral-flutter`, so only the inactive Astral project was cleaned with Flutter's owning command.
Class: storage-cleanup active-project-identification flutter simulator bundle-id

## L-001 — Pub Cache Is Not Disposable
Date: 2026-04-30 | Repo: mac-cleaner
Symptom: Flutter/Dart builds and hot restart fail because packages and globally activated tools disappear from `~/.pub-cache`.
Root cause: The scanner classified `~/.pub-cache` as a cleanable review-required cache, so bulk cleanup could remove the shared Pub package store during active Flutter work.
Fix: Mark `Dart pub cache` report-only in `scanner.py`, add a hard `.pub-cache` cleanup guard, and cover both with scanner regression tests.
Class: storage-cleanup pub-cache flutter dart package-store

<!-- Append new entries above this line, newest at top. -->
<!-- Use sequential IDs: L-001, L-002, … -->
