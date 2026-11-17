# Mac Cleaner — Click-through QA Handoff

Date: 2026-11-16 · Repo: `/Users/danilulmashev/Documents/GitHub/mac-cleaner-full-rust-rewrite`
Branch: `feature/full-rust-rewrite` · Stream: `.platform/work/usable-cleanup.md`

The owner explicitly asked Codex to commit the complete working tree and push this branch on
2026-11-16 so work can continue on another computer. Start from the pushed branch; do not assume
the app installed on the next Mac contains this source revision.

You are driving the real native app with computer use. Goal: find out **what doesn't work and
what is confusing** for a normal user who wants to free disk space. Earlier sessions could not
click (no Accessibility permission), so **nothing below has been seen on screen yet**.

## 1. What changed since the last QA (why this pass matters)

The previous app could analyse storage but could not free anything: the safe scan always returned
0 items and every row only offered "Reveal in Finder". This session changed that:

- **Rust eligibility** (`crates/cleaner-policy/src/catalog.rs`, `crates/cleaner-fs/src/{scanner,metadata}.rs`):
  new cleanup targets: project `node_modules`, Rust `target`, `.dart_tool`, Flutter `build`
  (only directly next to `package.json` / `Cargo.toml` / `pubspec.yaml` under `~/Documents/GitHub`,
  one or two levels deep), plus the Yarn, npm, npx and Puppeteer caches. Project sentinels such as
  `Podfile` and `.git` now block a folder only at its top two levels, and symlinks or hard links are
  allowed inside generated folders. Xcode builds used within 14 days are now listed with that reason.
- **Read-only scan on this Mac:** 11 items, **14.24 GB** (was 0).
- **UI**: the "Cleanup" tab is now **"Free Up Space"** (second in the sidebar), rewritten in `CleanupView.swift`:
  three groups (download caches, project dependencies and builds, old Xcode builds), checkboxes,
  last-changed and rebuild-cost text, badges (IN USE TODAY / THIS APP'S BUILD / UNUSED 30+ DAYS),
  a bottom bar with **Clean Selected…**, and a dialog offering *Move to Recovery* or
  *Delete permanently*. An **Empty Recovery** button is on Free Up Space and Recovery.
  A "Managed by other apps" list covers Docker, Xcode device support and similar.
- **Reopen fix** (`MacCleanerApp.swift`): the Dock icon and `open -a` recreate a closed main window.
- **Critical bug found and fixed in code:** every engine call (including the app's 5-second status
  poll) marked in-flight plans as *failed*. Any clean taking more than ~5 s (big `node_modules`) failed
  with `invalid plan transition … expected Executing, requested Completed`. A real move could succeed
  on disk but lose its Recovery record. Fix in `crates/cleaner-service/src/lib.rs` (`try_mutation_lock`):
  recovery of interrupted plans runs only when nobody holds `mutation.lock`.
  **Status: fixed and built, NOT yet installed or verified.** The installed app still has the old engine.

Rust: `cargo fmt`, `clippy -D warnings` and `cargo test --workspace` are green (8 new tests in
`crates/cleaner-fs/tests/generated_data.rs`). `xcodebuild test` reports "Could not launch
MacCleanerNativeTests"; not yet determined whether that predates these changes.

## 1.1 Where this session stopped

- The combined Swift/Rust source builds, but the newest bundled engine with the mutation-lock fix
  has not received the click-through in section 4.
- Codex could take screenshots and run the engine/MCP, but macOS denied it Assistive Access
  (`System Events` error `-25211`), so it could not click the UI honestly.
- Claude had computer-use access and prepared this script, but the owner chose to continue on
  another computer before the full pass was completed.
- No real cleanup was executed. A guarded scan was run; it found 11 review-required candidates.
  Active Mac Cleaner, TakeCare and Vibe Bloom build data was deliberately retained.
- No old simulator test recordings qualified under the 72-hour rule. Xcode data was recent and
  retained. Docker/GreyBox data was never changed.

## 1.2 MCP, CPU, memory and missing-disk diagnosis

The bundled read-only engine/MCP worked. During the last diagnostic it reported:

- whole-Mac CPU around 90% (`critical`), memory pressure `critical`, and about 22 GB swap in use;
- `/System/Volumes/VM` occupied 24 GB, including twelve new 1 GB swapfiles created while concurrent
  TypeScript, Jest, webpack, visual-regression Chrome and Docker jobs were running;
- from the 12:56 PM storage baseline, volume usage grew 14.79 GB: 1.96 GB was measured build/cache
  growth and 12.82 GB was the accounting gap, which direct inspection identified as swap;
- Docker's virtual disk was about 20.76 GB but grew only about 6.5 MB during the interval. It was
  not the cause and contains the protected GreyBox local PostgreSQL data;
- Mac Cleaner's own target grew about 1.4 GB, TakeCare node_modules grew about 426 MB, and Codex
  session history grew about 123 MB.

This exposed a product gap: Storage reports system swap as an unexplained accounting gap. A future
change should show a read-only `System swap` row, connect it to memory pressure, and explain that
users must reduce the workload or restart rather than delete swapfiles manually. Never raw-delete
`/System/Volumes/VM` or `Docker.raw`.

The active workload was not killed because repository safety rules protect developer jobs. On the
next computer, reproduce pressure only intentionally; do not terminate agent/test work without an
explicit owner decision.

## 2. Setup (do this first)

On the other computer:

```bash
git fetch origin
git switch feature/full-rust-rewrite
git pull --ff-only origin feature/full-rust-rewrite
```

Then build and install. Do not copy the old installed app between computers.

The new build is at `target/native-derived/Build/Products/Release/Mac Cleaner.app`. Install it.
This replaces the installed app; the owner asked for this QA, but confirm with them if unsure.

```bash
cd /Users/danilulmashev/Documents/GitHub/mac-cleaner-full-rust-rewrite
osascript -e 'tell application "Mac Cleaner" to quit'
# wait until the app process has exited:
while pgrep -f "Mac Cleaner.app/Contents/MacOS" >/dev/null; do sleep 0.5; done
/usr/bin/ditto 'target/native-derived/Build/Products/Release/Mac Cleaner.app' \
  '/Users/danilulmashev/Applications/Mac Cleaner.app'
open '/Users/danilulmashev/Applications/Mac Cleaner.app'
```

If the build is missing or stale: `bash ./scripts/build-universal.sh` (about 3–5 minutes).

## 3. Safety rules (override everything)

- **Do not perform a real clean of anything except the throwaway fixture in step 4.0.** Do not
  move or delete `node_modules`, Rust `target`, Yarn, npm or other real caches unless the owner says so in chat.
- Never touch Docker data, Xcode, simulators, SDKs, `.git`, sources, or the GreyBox database.
- Don't kill developer processes. The complete handoff state, including the two owner-approved QA
  file deletions, is already part of the pushed branch; do not recreate those deleted files.
- "Validate" or dry-run paths are fine. Cancel every destructive confirmation dialog you open,
  except on the fixture.

## 4. Click-through script

Record for **every step**: what you clicked, what you saw (screenshot), and pass/fail. Also record
anything a non-engineer wouldn't understand.

### 4.0 Create a throwaway fixture (the only thing you may really clean)
```bash
mkdir -p ~/Documents/GitHub/zz-qa-fixture/node_modules/left-pad
echo '{}' > ~/Documents/GitHub/zz-qa-fixture/package.json
mkfile 300m ~/Documents/GitHub/zz-qa-fixture/node_modules/left-pad/blob.bin
```
Expected: after a rescan, a 300 MB "zz-qa-fixture node_modules" row appears under *Project dependencies and builds*.

### 4.1 Launch and window
1. The app opens with the sidebar order Overview, **Free Up Space**, Storage, Recovery, History, Agents.
2. Close the main window (red button). Click the Dock icon → **the window must come back**.
3. Close it again. Run `open -a "/Users/danilulmashev/Applications/Mac Cleaner.app"` → it must come back.
4. Close it again. Menu bar item → **Open Mac Cleaner** → it must come back.
5. Is there any shadow, oval or pill glitch on the title bar, sidebar or status bar?

### 4.2 Free Up Space: first impression (most important)
1. Open **Free Up Space**. Before reading the details, can you answer: *how much can I free, and what do I click?*
2. Summary tiles: **Can be freed** ≈ 14 GB (+0.3 GB fixture), **Selected** 0, **In Recovery** 0.
3. While scanning, is there visible progress, or does it look frozen? Scans take about 20–35 s.
4. Groups appear in this order: caches, projects, Xcode. Each shows a size and a Select all link.
5. Rows: the name is readable (project name, not a hash), size is right-aligned, last-changed and rebuild cost make sense.
6. Badges: `mac-cleaner-full-rust-rewrite` Rust target shows **THIS APP'S BUILD**. Anything changed today shows **IN USE TODAY**.
7. Click the magnifier on a row → Finder opens with that folder selected. Nothing changes.
8. Expand "N item(s) not offered…" → reasons such as *used within the last 14 days* are understandable.
9. "Managed by other apps": Docker ~20 GB → **Open Docker** only opens Docker. Xcode rows → Open Xcode.

### 4.3 Selection and confirmation (do not confirm on real data)
1. Tick two cache rows. The bottom bar updates to *2 selected · X GB*. The Selected tile matches.
2. Click a row's text (not the checkbox) → it toggles.
3. Select all / Deselect all per group works.
4. Select an IN USE TODAY item → the bottom bar shows an amber warning.
5. Click **Clean Selected…** → the dialog lists the items and sizes and offers *Move to Recovery (can undo)*,
   *Delete permanently and free X now*, and Cancel. Is the difference clear? **Click Cancel.**
6. Deselect everything; the button is disabled.

### 4.4 Real clean, fixture only
1. Select **only** the zz-qa-fixture row. Clean Selected… → **Move to Recovery**.
2. The status bar shows progress text. Afterwards the row disappears, **In Recovery** ≈ 300 MB,
   and `~/Documents/GitHub/zz-qa-fixture/package.json` still exists.
3. **Recovery** tab: the entry is listed. Click **Restore** → confirm → the folder is back in the project.
4. Rescan, select the fixture again → Clean Selected… → **Delete permanently**. Free space grows by ~300 MB,
   Recovery stays 0, and History shows a receipt.
5. Repeat with a large fixture (`mkfile 2g`, plus many small files) to make sure the
   "invalid plan transition" bug is gone for slow cleans. You can create many small files with
   `for i in $(seq 1 20000); do echo x > ~/Documents/GitHub/zz-qa-fixture/node_modules/left-pad/f$i; done`.
6. Recovery → **Empty Recovery** on a fixture entry: the dialog explains the consequences, the action works,
   and the total updates.
7. Cleanup: `rm -rf ~/Documents/GitHub/zz-qa-fixture` (only this exact path).

### 4.5 Overview, Storage, History, Agents, menu bar (quick pass)
- Overview: the quick win says "X can be freed" → **Free up space** navigates correctly. RAM and CPU buttons
  still only open Activity Monitor (known, deferred; note how unhelpful this feels at *critical* memory pressure).
- Storage: Analyze Again, filter, Set Baseline, Copy Report all work. Does Storage still feel like a
  duplicate of Free Up Space? Is the "incomplete measurements" warning (`~/Library/Caches` not
  readable without Full Disk Access) noticeable?
- History: receipts from 4.4 appear with size and time.
- Agents: MCP status and tool list are visible. Read-only tools work.
- Menu bar: icons and colours, popover contrast, Open Mac Cleaner, Analyze Storage, refresh, Quit.

## 5. Optional engine-level check (no screen needed)
Dry-run validate every candidate. This runs every safety re-check and moves nothing:
```bash
python3 - <<'EOF'
import json, subprocess
E = './target/release/mac-cleaner-engine'
scan = json.loads(subprocess.run([E, 'scan'], capture_output=True, text=True).stdout)
scan = scan.get('data', scan)
for c in scan['candidates']:
    p = json.loads(subprocess.run([E, 'prepare-clean', '--candidate', c['id'], '--dry-run'],
                                  capture_output=True, text=True).stdout)
    p = p.get('data', p)
    x = subprocess.run([E, 'execute-clean', p['id']], capture_output=True, text=True)
    print('OK  ' if x.returncode == 0 else 'FAIL', c['lexical_path'], x.stderr[:160])
EOF
```
Before the fix: 9 OK, 2 FAIL (both `node_modules`). **Expected after installing: 11 OK.**

## 6. Known open items (don't re-report, but do note their impact)
- RAM and CPU actions are advisory only. A "Quit app" for apps that aren't developer tools needs an owner decision.
- Docker 20.8 GB has no breakdown or prune flow.
- No Full Disk Access onboarding, so `~/Library/Caches` is unmeasured.
- Scans re-measure everything (about 20–35 s), and each clean rescans before planning.
- Hard-linked files are counted at full size; freed space can be slightly lower than shown.

## 7. Report back
1. A pass/fail table for 4.1–4.5 and section 5.
2. Screenshots: Free Up Space top, the groups, the confirmation dialog, Recovery with the fixture, the menu bar popover.
3. Every bug with exact reproduction steps.
4. Every confusing label, number or action order, with a suggested rewrite.
5. Mutations performed (fixture only).
6. Verdict: can a normal user answer *"what can I safely clean, how much will it free, what will it cost me?"* in under 30 seconds?
