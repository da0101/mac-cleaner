---
domain_id: dom-safe-cleanup-policy
slug: safe-cleanup-policy
status: active
repo_ids: [mac-cleaner]
created_at: 2026-08-10
updated_at: 2026-11-16
---

# safe-cleanup-policy

The cleanup policy separates discovery from authorization and treats unknown filesystem state as protected.

## Safety tiers

- `safe-auto`: empty for the initial Rust release; future entries require recovery and adversarial tests.
- `review-required`: exact, known generated artifacts that require an explicit plan and native confirmation.
- `protected`: source repositories, application state, toolchains, credentials, databases, user content, unknown paths, and protected ancestors/descendants.

## Authorization contract

1. Rust discovers a candidate from a compiled catalog; clients cannot submit paths.
2. Planning captures target ID, lexical and resolved path, device/inode, kind, size, mtime, catalog version, and expiry.
3. Approval is single-use and bound to the exact immutable plan.
4. Execution reacquires a cross-process mutation lock and revalidates containment and identity immediately before mutation.
5. Changed, linked, mounted, protected, expired, replayed, or unknown targets fail closed.
6. Every transition and completed item is journaled before and after mutation; crash recovery never resumes deletion automatically.
7. MCP uses the same contract: prepare tools return an exact plan and plan-bound phrase; destructive execution tools reject paths and run only after explicit confirmation. Recoverable cleanup and permanent purge remain separate actions.

Read-only storage hotspots are explanatory only. Their size, label, or recommendation cannot be converted into a cleanup plan unless the path is independently rediscovered by the versioned cleanup catalog and passes the normal planning contract.

## Permanent protections

- Repository roots, worktrees, submodules, `.git` directories, and `.git` indirection files.
- GitHub Desktop Application Support, preferences, authentication, and Keychain state.
- Flutter/Dart/Android/Xcode SDKs and toolchains; package stores and global activation metadata.
- Gradle configuration, init scripts, toolchains, wrapper files, and project manifests.
- Source, lockfiles, signing files, credentials, databases, documents, photos, and unknown paths.

## Test boundary

Destructive tests run only inside an isolated temporary fake home or disposable APFS image containing outside-canary files. They must never point at a real home directory.

## Key files

- `crates/cleaner-policy/src/`
- `crates/cleaner-fs/src/scanner.rs`
- `crates/cleaner-fs/src/executor.rs`
- `crates/cleaner-service/src/lib.rs`
- `crates/cleaner-fs/tests/safety.rs`
- `crates/cleaner-service/tests/service_flow.rs`
