# Rescan protocol — {{PROJECT_NAME}}

**Read this file when the user says** "update the platform", "rescan", "refresh
context", "you're out of date", or similar — anything that means "re-read the
codebase and bring `.platform/` up to date."

This is the incremental refresh protocol. It is faster than a full activation
(no user interview) and is safe to re-run at any time — it only adds or updates,
never destroys accumulated knowledge.

Run `ab rescan` first for a staleness summary, then follow the steps below.

---

## Step 1 — Quick scan (~2 minutes, read-only)

Read the actual project state. Do not use memory from this session — look at the
files themselves:

- **Directory tree** (2 levels, skip `node_modules/`, `.git/`, `build/`, `dist/`,
  `Pods/`, `.gradle/`, `target/`, `venv/`, `__pycache__/`)
- **Recent git history** — `git log --oneline -30` to see what has been built
  since the last scan
- **New or changed source directories** — anything that wasn't in
  `architecture.md` or `domains/` already
- **`package.json` / `pyproject.toml` / manifest files** — check for new
  dependencies that signal new capabilities or integrations
- **`conventions/`** — skim existing files to understand the current rules before
  updating them

From this, build a mental diff: *what exists in the codebase that `.platform/`
does not yet describe?*

## Step 2 — Compare vs existing context

Open each content file and note the gap:

| File | Common staleness signal |
|---|---|
| `STATUS.md` | Features marked "planned" that are now shipped; missing new features |
| `architecture.md` | New services, libs, or integrations not listed; changed data flow |
| `domains/` | Features that exist in code but have no domain file; domains whose key files have changed |
| `conventions/` | New patterns used consistently in recent commits that aren't documented |

Skip a file entirely if the scan reveals no material change.

## Step 3 — Update (additive-first, targeted)

Work through the gaps found in Step 2. For each file:

### `STATUS.md`

Update feature rows to reflect current reality:
- Move shipped features from "planned/in progress" to "live"
- Add newly discovered features in the right status bucket
- Update the "immediate priorities" section if the git log signals a shift in focus
- Do **not** remove existing rows — mark them stale with `<!-- stale: reason -->`
  if they no longer apply

### `architecture.md`

Add new components, services, or integrations you found. If a section is
materially wrong (e.g., wrong database listed), correct it — but leave a comment
explaining what changed. Keep each section under one screen.

### `domains/`

- **New domain:** create `.platform/domains/<slug>.md` from
  `.platform/domains/TEMPLATE.md`. Fill in the metadata block and key files.
- **Existing domain:** update `key_files`, `entry_points`, and the description
  if the feature has grown. Do not remove context — append with a note.

### `conventions/`

- **New patterns detected:** add rules to the relevant stack file, or create a
  new file if a new stack was added.
- **Outdated rules:** annotate with `<!-- updated: reason -->`, do not delete —
  the history is useful context.

### `memory/log.md`

Append exactly one line:

```
<YYYY-MM-DD> — ab rescan — <one-sentence summary of what changed>
```

## Step 4 — Report

After updating, write a short summary in chat:

```
## Rescan complete

**Updated:** STATUS.md (3 features promoted to live), domains/checkout.md (new),
conventions/api.md (added rate-limit pattern)

**No change needed:** architecture.md (matches current code), repos.md (use ab bootstrap)

**Skipped (protected):** decisions.md, ACTIVE.md, BRIEF.md, learnings.md,
gotchas.md, playbook.md
```

---

## Protected files — never modified by rescan

These files accumulate knowledge the agent cannot safely regenerate. Only a human
or an explicit user-directed write should touch them:

- `memory/decisions.md` — locked architectural decisions
- `work/ACTIVE.md` — live stream registry
- `work/BRIEF.md` — current stream brief
- `memory/learnings.md` — post-stream lessons
- `memory/gotchas.md` — hard-won warnings
- `memory/playbook.md` — reusable recipes
- `repos.md` — use `ab bootstrap` to refresh this
