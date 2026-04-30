# Feature Brief — mac-cleaner

**Feature:** Pub cache protection
**Status:** closure-review
**Stream file:** `work/pub-cache-protection.md`

## What we're building

Prevent mac-cleaner from deleting Dart/Flutter's global Pub package cache while keeping the storage usage visible.

## Why

`~/.pub-cache` is a shared development package store. Removing it during active Flutter work can break builds, hot restart, and globally activated tools.

## Done looks like

- `~/.pub-cache` is report-only/non-cleanable.
- Cleanup code refuses `.pub-cache` even if passed stale cleanable metadata.
- Scanner regression tests cover the policy.

## Relevant context

- `.platform/domains/storage-cleanup.md`
- `.platform/conventions/security.md`
- `.platform/conventions/testing.md`
