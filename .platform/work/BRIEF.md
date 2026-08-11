# Feature Brief — mac-cleaner

**Feature:** Full Rust application rewrite
**Status:** in-progress
**Stream file:** `work/full-rust-rewrite.md`

## What we're building

Replace the Python cleaner, localhost dashboard, and menu-bar scripts with a release-grade Rust macOS application and CLI built around a fail-closed cleanup policy.

## Why

The existing cleanup surfaces can delete targets based on a broad `cleanable` flag without enforcing safety tiers. The replacement must protect repositories, application state, development toolchains, and unknown data by construction.

## Done looks like

- The shipped app and CLI require no Python or Node runtime.
- Cleanup execution accepts only a fresh, immutable, confirmed server-owned plan.
- Automatic storage cleanup begins with an empty allowlist.
- The native app owns monitoring, menu bar, login startup, settings/history, and universal macOS packaging.

## Relevant context

- `.platform/domains/rust-application.md`
- `.platform/domains/safe-cleanup-policy.md`
- `.platform/domains/memory-optimization.md`
- `.platform/domains/local-dashboard.md`
- `.platform/conventions/testing.md`
