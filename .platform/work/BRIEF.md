# Feature Brief — mac-cleaner

**Feature:** Native SwiftUI macOS shell with Rust backend
**Status:** closure-review
**Stream file:** `work/native-swiftui-shell.md`

## What we're building

Replace the Tauri/React shell with a pure SwiftUI/AppKit macOS application while retaining the Rust engine, policy, storage history, and persistence.

## Why

The product is macOS-only and should use native controls, lifecycle, accessibility, and packaging without a web runtime or third-party UI framework.

## Done looks like

- A native SwiftUI `.app` provides feature parity with the current desktop shell.
- A bundled Rust helper exposes only typed, policy-gated operations and identifiers.
- Storage forensics remains compact, searchable, historical, and copyable.
- Tauri, React, Python, and the web frontend are removed after native verification.
- The Rust engine, Swift app, universal packaging, and local safety gates pass; App Store sandbox/signing is a separate release follow-up.

## Relevant context

- `.platform/domains/native-macos-shell.md`
- `.platform/domains/storage-forensics.md`
- `.platform/domains/rust-application.md`
- `.platform/domains/safe-cleanup-policy.md`
- `.platform/conventions/testing.md`
