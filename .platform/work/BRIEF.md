# Feature Brief — mac-cleaner

**Feature:** Menu bar and dashboard memory display
**Status:** in-progress
**Stream file:** `work/menubar-memory-widget.md`

## What we're building

Fix display regressions in the local dashboard startup banner, dashboard memory stat card, and macOS menu bar RAM chip.

## Why

The dashboard and menu bar should present the same free/available RAM model so low raw free memory is visible without hiding reusable memory.

## Done looks like

- Startup banner brands the command as Mac Cleaner without the internal "SERVER" label.
- Dashboard memory card shows raw free RAM and available RAM together.
- Dashboard free RAM color thresholds match the menu bar widget.
- Menu bar RAM icon is drawn as a horizontal module.

## Relevant context

- `.platform/domains/memory-optimization.md`
- `.platform/domains/local-dashboard.md`
- `.platform/conventions/testing.md`
