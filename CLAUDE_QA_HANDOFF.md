# Claude QA Handoff — Mac Cleaner

> **Superseded:** Continue with `QA_CLICKTHROUGH_HANDOFF.md`. That document reflects the later
> usable-cleanup implementation, Rust eligibility changes, mutation-lock fix, MCP/swap diagnosis,
> and the cross-computer continuation state. This earlier document is retained as historical context.

Date: 2026-11-16

Repository: `/Users/danilulmashev/Documents/GitHub/mac-cleaner-full-rust-rewrite`

Branch: `feature/full-rust-rewrite`
HEAD: `ed387ec chore: record native cleaner checkpoint`

## Mission

Manually drive and QA the installed native Mac Cleaner app. Focus first on the new guided Cleanup flow: the previous UI said `16.21 GB needs review`, but a safe scan found zero eligible files and gave the user no clear next step.

The product must make the distinction explicit:

- **Ready now** means files that passed Rust catalog, age, identity, and safety rules.
- **Rebuildable data** is measured evidence that still requires a project-by-project decision. It is not approved deletion.
- **Held for recovery** is quarantined data that still occupies disk space.

Do not delete anything as part of QA. Test discovery, navigation, labels, read-only inspection, and confirmation boundaries.

## Safety rules that override everything

- Do not delete repositories, worktrees, source files, credentials, databases, or unknown data.
- Do not remove Xcode, iOS runtimes, simulators, Android SDKs, Android emulator images, Flutter, Rust, Homebrew, or other developer toolchains.
- Do not remove or reset the GreyBox Docker PostgreSQL database, image, container, or volume.
- Do not clean the active Mac Cleaner build output during this QA.
- Do not clean Vibe Bloom; it is active user work.
- Do not kill Codex, Claude, editors, terminals, compilers, VMs, or developer processes.
- Do not run destructive MCP tools. Approval and revalidation must remain visible.
- Do not restore, stage, or commit the two user-owned deleted QA files listed under **Working tree**.

## Current implementation state

The last release build passed and was copied to:

`/Users/danilulmashev/Applications/Mac Cleaner.app`

The app process and bundled MCP processes are running. The menu bar currently shows live RAM, CPU, and free SSD values.

Latest work is intentionally uncommitted:

- `apps/mac-cleaner-macos/Sources/OverviewView.swift`
  - Adds Quick Wins for cleanup, memory pressure, and CPU pressure.
  - Aligns the process table.
  - Explains CPU units: whole-Mac CPU is normalized to 0–100%; process CPU follows macOS semantics where 100% equals one logical core, so a process may exceed 100%.
- `apps/mac-cleaner-macos/Sources/CleanupView.swift`
  - Replaces the ambiguous summary with Ready now / Rebuildable data / Held for recovery.
  - Adds a guided explanation when zero files qualify.
  - Adds an exact location-by-location breakdown of the rebuildable total.
  - Marks the current Mac Cleaner build as `ACTIVE BUILD · KEEP`.
  - Marks Xcode DerivedData as `NO OLD ITEMS QUALIFY` when the 14-day rule returns zero candidates.
  - Marks tiny locations as `LOW VALUE`.
  - Gives manual locations a `Reveal in Finder` action instead of a delete action.
- `apps/mac-cleaner-macos/Sources/AppModel.swift`
  - Adds Finder reveal support for the exact measured location.

The release build completed successfully after these edits. The installed app contains this build, but the new Cleanup page has not yet received a visual click-through because the Codex host lacks macOS Accessibility permission.

## Working tree — preserve exactly

Expected `git status --short`:

```text
 D .platform/work/qa/full-rust-rewrite-execution-journal.md
 D .platform/work/qa/full-rust-rewrite-manual-qa.md
 M apps/mac-cleaner-macos/Sources/AppModel.swift
 M apps/mac-cleaner-macos/Sources/CleanupView.swift
 M apps/mac-cleaner-macos/Sources/OverviewView.swift
?? CLAUDE_QA_HANDOFF.md
```

The two deletions are pre-existing user-owned changes. Do not restore or commit them.

Do not commit the new implementation unless the user explicitly asks after QA.

## Important live storage facts

The last read-only Rust analysis reported approximately `16.21 GB` of rebuildable/reviewable data and zero safe direct-cleanup candidates.

Largest contributors:

| Location | Approx. size | Expected treatment |
|---|---:|---|
| Current Mac Cleaner Rust `target` | 6.29 GB | Active build; keep |
| Xcode DerivedData | 2.84 GB | No item currently passes the 14-day rule |
| `takecare-frontend/node_modules` | 2.72 GB | Manual project decision; do not delete in QA |
| `takecare-frontend-tk-2803-manual-workflow-start/node_modules` | 2.17 GB | Active/manual work; do not delete in QA |
| Yarn Berry cache | 1.48 GB | Manual/tool-owned decision |
| Vibe Bloom Rust `target` | 336 MB | Active project; keep |
| Puppeteer cache | 205 MB | Manual/tool-owned decision |
| npx cache | 74 MB | Manual/tool-owned decision |
| npm cache | 59 MB | Manual/tool-owned decision |
| takecare-mobile `.dart_tool` | 39 MB | Low value; keep during QA |

Docker is about `20.76 GB`, but it is tool-managed and is not part of the `16.21 GB` rebuildable total. It contains protected GreyBox local database data and must not be raw-deleted.

## How to open the app

1. Confirm `/Users/danilulmashev/Applications/Mac Cleaner.app` is running.
2. Click the RAM / CPU / SSD item in the macOS menu bar.
3. Click **Open Mac Cleaner**.
4. If the main window is already visible, bring it forward normally.

High-priority regression to test: after closing the main window, running `open -a "/Users/danilulmashev/Applications/Mac Cleaner.app"` did not bring the window back in the Codex session. Determine whether the menu-bar **Open Mac Cleaner** action always restores it. Record this as a bug if reopen behavior is inconsistent.

## Manual QA checklist

### 1. Launch and persistence

- [ ] App launches without a missing-engine warning.
- [ ] Main window opens or can be restored from the menu bar.
- [ ] App starts a storage/system refresh automatically.
- [ ] Previously granted folders do not prompt repeatedly.
- [ ] System / Light / Dark appearance selection persists after relaunch.
- [ ] Sidebar, title bar, refresh button, and status bar have no shadow/oval/pill glitches.

### 2. Overview

- [ ] Storage, memory, swap, and whole-Mac CPU values are readable and update.
- [ ] CPU card makes clear that its percentage is the whole Mac, normalized to 100%.
- [ ] **Quick wins** appears before the full process table.
- [ ] **Review cleanup** navigates to Cleanup.
- [ ] **Inspect memory** and **Inspect CPU** open Activity Monitor or the intended inspection surface.
- [ ] Process rows align under Memory / CPU / Action.
- [ ] Process CPU header says `100% = 1 core`.
- [ ] Values over 100% are visually valid and not presented as an error.
- [ ] Protected developer processes do not offer a quit/kill action.

### 3. Cleanup — primary focus

- [ ] Summary shows **Ready now: 0**, not `16.21 GB ready`.
- [ ] Summary shows approximately **16.21 GB Rebuildable data** and says it is measured evidence, not approved deletion.
- [ ] Guided card says nothing currently qualifies for automatic/direct cleanup.
- [ ] Guided card explains that the safe scan found zero items passing catalog + age rules.
- [ ] **Run eligibility scan** works and remains non-destructive.
- [ ] **Where the rebuildable total comes from** is visible without guessing what to click next.
- [ ] Rows are sorted largest first and show name, exact path, size, state, explanation, and one clear action.
- [ ] Current Mac Cleaner `target` is `ACTIVE BUILD · KEEP` and has no delete action.
- [ ] Xcode DerivedData shows `NO OLD ITEMS QUALIFY` after the empty scan.
- [ ] Low-value entries are labeled `LOW VALUE`.
- [ ] Manual entries say `MANUAL DECISION` and only offer **Reveal in Finder**.
- [ ] **Reveal in Finder** selects the exact measured folder and performs no mutation.
- [ ] The lower **Recommended actions** section is visually secondary to the guided breakdown.
- [ ] Docker action only opens Docker; it never deletes Docker files directly.
- [ ] Empty **Approved file cleanup** state explains why nothing qualifies.
- [ ] No button implies that all 16.21 GB can be safely removed in one click.

### 4. Recovery and History

- [ ] Recovery empty state explains that quarantined files would appear there.
- [ ] Recovery does not claim to free space until permanent purge.
- [ ] History shows completed Rust/tool maintenance receipts with size and timestamp.
- [ ] Relaunch preserves history.

### 5. Agents / MCP

- [ ] Agents page explains how Codex and Claude connect to the bundled MCP.
- [ ] MCP status is healthy and the tool list is visible.
- [ ] Read-only storage/system tools work from Claude.
- [ ] Cleanup tools return a typed plan first and require explicit confirmation.
- [ ] MCP accepts no arbitrary path or shell text.
- [ ] Do not execute a destructive cleanup during QA.

### 6. Menu bar

- [ ] RAM uses a memory icon, CPU uses a CPU icon, and SSD uses a storage icon.
- [ ] Each icon changes color according to its own pressure state.
- [ ] Popover values and text have sufficient contrast.
- [ ] CPU pressure and top CPU processes are understandable.
- [ ] **Open Mac Cleaner**, **Analyze Storage**, refresh, and Quit work.
- [ ] The menu-bar item remains compact at common resolutions.

## Build command

From the repository root:

```bash
MAC_CLEANER_ENGINE_PATH="$PWD/target/universal-macos/mac-cleaner-engine" \
MAC_CLEANER_MCP_PATH="$PWD/target/universal-macos/mac-cleaner-mcp" \
xcodebuild -quiet \
  -project apps/mac-cleaner-macos/MacCleanerNative.xcodeproj \
  -scheme MacCleanerNative \
  -configuration Release \
  -derivedDataPath target/native-derived \
  -destination 'generic/platform=macOS' \
  ARCHS='arm64 x86_64' \
  ONLY_ACTIVE_ARCH=NO \
  CODE_SIGNING_ALLOWED=NO \
  build
```

Install the verified build only if source changes are required:

```bash
osascript -e 'tell application "Mac Cleaner" to quit'
/usr/bin/ditto \
  'target/native-derived/Build/Products/Release/Mac Cleaner.app' \
  '/Users/danilulmashev/Applications/Mac Cleaner.app'
open '/Users/danilulmashev/Applications/Mac Cleaner.app'
```

Do not replace the app while a storage analysis helper is still writing state. Wait for the app process to exit first.

## Automated checks already completed

- Universal Release `xcodebuild` passed after the Cleanup and Overview edits.
- `git diff --check` passed.
- Installed app launches its native binary and bundled Rust helpers.
- No cleanup or deletion was executed during this work.

## What to report back

Return a concise QA report with:

1. Pass/fail for each section above.
2. Screenshots for Cleanup top, rebuildable-location list, Overview process table, Agents, and menu-bar popover.
3. Every button clicked and its observed result.
4. Any confusing copy or action hierarchy.
5. Any mutation attempted (ideally none).
6. Exact reproduction steps for failures.
7. Whether the user can answer, without explanation from an engineer: “What can I safely clean right now, how much will it free, and what will it cost to rebuild?”

Do not commit, push, close the Agentboard stream, or delete data without the user's explicit instruction.
