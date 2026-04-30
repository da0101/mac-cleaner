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

## L-001 — Pub Cache Is Not Disposable
Date: 2026-04-30 | Repo: mac-cleaner
Symptom: Flutter/Dart builds and hot restart fail because packages and globally activated tools disappear from `~/.pub-cache`.
Root cause: The scanner classified `~/.pub-cache` as a cleanable review-required cache, so bulk cleanup could remove the shared Pub package store during active Flutter work.
Fix: Mark `Dart pub cache` report-only in `scanner.py`, add a hard `.pub-cache` cleanup guard, and cover both with scanner regression tests.
Class: storage-cleanup pub-cache flutter dart package-store

<!-- Append new entries above this line, newest at top. -->
<!-- Use sequential IDs: L-001, L-002, … -->
