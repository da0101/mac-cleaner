# Workflow — The 6-Stage Inline Workflow

> **Audience:** any AI agent (Claude Code, Codex CLI, Gemini CLI) or human working on this project.
> **Goal:** move fast on small tasks, think carefully on big ones, and never bureaucratize small work.

---

## The 6 stages

```
Triage → Interview → Research → Propose → Execute → Verify + Learn
```

Each stage has a clear entry condition and a clear exit condition. Skip stages that don't apply.

### Silicon Valley product mindset

For every product, feature, cleanup, or technical decision, think like a
best-in-class Silicon Valley product team: user-obsessed, future-facing,
innovative, craft-driven, fast, and rigorous. Raise the bar beyond basic task
completion by asking what would make the result differentiated, durable,
scalable, delightful to use, and credible against leading technology products.

This mindset is not permission for vague hype or scope creep. Ambition must be
translated into the smallest coherent slice, explicit tradeoffs, strong
execution quality, tests, rollback thinking, and human approval for any scope
change. PM work must sharpen user value, differentiation, and success criteria;
engineering work must turn that ambition into maintainable, performant,
well-integrated implementation.

### New stream intake contract

When a user request is not already tracked in `work/ACTIVE.md` and should become a new stream, every provider follows this strict order:

```
Detect new stream → Register → Branch + local env → Clarify → Research → Plan → Human approval → Execute → Verify + Learn
```

- **Research is always required for new streams.** Scale the depth to the task: small/low-risk streams may use a compact local + targeted web check, while medium+ or risky streams need the full research pass.
- **Research must be specific to the work.** Cover the problem, comparable external examples or prior art, current patterns, implementation techniques, best practices, local code/docs, caveats, and a recommendation.
- **Planning follows research.** Plans must include development phases, complexity, risk mitigation, alternatives considered, files to touch, tests, rollback path, and clarifying questions if anything is still ambiguous.
- **Work starts on a stream branch.** Before implementation for feature, bugfix, or hotfix streams, create or switch to a `feature/<slug>` or `bugfix/<slug>` branch in each touched repo (from `develop`; hotfix from `master` only when the user explicitly says "hotfix"). Worktrees are on demand, not the default — see 1c.
- **Local environment is prepared before coding.** In every checkout you will work from, install the repo's development dependencies and identify the local dev command plus localhost port(s) before implementation or manual QA planning.
- **Human-in-the-loop is mandatory.** The agent must present the research-backed plan and wait for human validation/approval before implementation starts. During implementation, the agent must pause for clarification when the approved plan no longer fits reality.
- **Implementation follows the approved plan.** Deviations are called out explicitly in chat and captured in the stream file via checkpoint/progress state when they affect scope, risk, or next action.

### 1. Triage

For every non-trivial task, state inline in chat:

- **Type:** bug / feature / refactor / chore / investigation / docs
- **Scope:** trivial / small / medium / large
- **Risk:** low / medium / high (blast radius + reversibility)

**Exit:** you know how big this is and how careful to be.

Trivial tasks (typo fix, rename, 1-line config change) skip directly to Stage 5.

### 1b. Register (mandatory for non-trivial tasks)

**Before doing anything else** — research, proposals, code — register the workstream:

1. **Check `work/ACTIVE.md`** — does this stream already exist? If yes, load the stream file and continue from where it left off. Do not create a duplicate.
2. **Check `.platform/domains/`** — does a domain file exist for **this specific concern**?
   - Ask: "Does an existing domain file fully describe the cross-repo touch-points for THIS concern (which files in which repos, what the API contract is, what breaks if this changes)?" A related-but-different domain file does NOT count.
   - If **yes and it's accurate**: read it, verify it's current, update if stale.
   - If **no, or the existing file only partially covers it**: create `.platform/domains/<name>.md` with the cross-layer touch-point inventory. Create it NOW, before the stream file.
   - **Common trap:** finding a domain file for a nearby feature (e.g. `menu-builder.md`) and treating it as sufficient for a different concern (e.g. subdomain routing). These are separate concerns and require separate domain files.
3. **Create `work/<stream-slug>.md`** from `work/TEMPLATE.md` — fill in the frontmatter metadata (`stream_id`, `slug`, `type`, `status`, `agent_owner`, `domain_slugs`, `repo_ids`, `base_branch`, `git_branch`, `created_at`, `updated_at`, `closure_approved`) before writing scope, done criteria, and next action. Keep `stream_id` canonical: `stream-<slug>`. Defaults: `base_branch: develop`, `git_branch: feature/<slug>` (or `bugfix/<slug>`), `closure_approved: false` — never set it `true` yourself; only the human flips it at closure.
4. **Add a row to `work/ACTIVE.md`** — slug / type / in-progress / agent / date.
5. **Update `work/BRIEF.md`** — set primary stream to this task; add domain file under "Relevant context".

**Why this is non-negotiable:** if the context is cleared, the computer restarts, or a different agent picks up the work, the stream file is the only way to resume. Zero registration = zero traceability = zero resumability. A workstream without a domain file has no focused context for the next agent.

Full protocol: `agents/work-tracking.md` § "Starting a new workstream".

**Exit:** domain file exists, stream file exists, `ACTIVE.md` has the row, `BRIEF.md` is current.

### 1c. Branch + local environment prep (worktree only on demand)

Before feature, bugfix, or hotfix implementation begins, put the work on its own branch:

1. **Determine stream kind and branch name**
   - Feature work: `feature/<stream-slug>` from `develop`
   - Bug fixes: `bugfix/<stream-slug>` from `develop`
   - Hotfixes: `hotfix/<stream-slug>` from `master` only when the user explicitly says this is a hotfix
   - If a repo uses a different production branch, use it only when the project docs or user explicitly override `master`.
2. **Create the branch in each touched repo's main checkout** (`git fetch origin && git switch -c feature/<stream-slug> origin/develop`). If the branch already exists, verify it points at the correct base and continue there.
3. **Use a worktree only on demand** — when the user asks, when two sessions must edit the same repo at the same time, or when the main checkout holds unrelated uncommitted work. Create it with `ab worktree new <stream-slug>` (add `--type bugfix|hotfix` as needed), record its path in the stream file, and remove it when the branch merges: `ab worktree status` lists leftovers, `ab worktree clean --apply` removes merged ones.
4. **Install development dependencies in the checkout you work from**
   - Use the repo's lockfile and toolchain: `npm ci`, `yarn install --frozen-lockfile`, `pnpm install --frozen-lockfile`, `uv sync`, `pip install -r requirements.txt`, `flutter pub get`, etc.
   - If dependency install fails, stop and surface the blocker instead of coding against a partially prepared environment.
5. **Identify local run commands and ports**
   - Read the repo docs/scripts/env/docker compose config to find the dev server command(s) and localhost port(s).
   - Record the command and port in the stream file under `## Worktree / Local environment`.

**Exit:** every touched repo is on its stream branch (worktree path recorded if one was created), dependencies installed or blocker reported, and known local run command(s)/localhost port(s) recorded.

### 2. Interview

**Only if requirements are ambiguous.** Ask 2–5 targeted questions. Do not ask "is my plan ready?" — use the plan-approval tool for that.

**Exit:** requirements are unambiguous.

### 3. Research

**Always for new streams; otherwise required for medium+ scope.** Parallelize:
- Subagent A: read existing code paths that touch the area
- Subagent B: web search / docs fetch (strict budget: 1 search + 2–3 fetches; smaller for low-risk streams, but do not skip it for new streams)
- Subagent C: check conventions/ and decisions.md for prior art

Synthesize in chat (≤300 words). Include the problem, comparable examples/prior art, implementation patterns, best practices, caveats, and recommendation. **Do not** write a research `.md` file.

**Exit:** you understand the area well enough to propose.

### 4. Propose

State a 5–10 bullet plan **inline in chat**. Include:
- Files to touch
- New files / deleted files
- Development phases
- Complexity assessment
- Test plan
- Risk factors
- Risk mitigations
- Alternatives considered
- Rollback path (for risky changes)
- Clarifying questions, if any requirement remains ambiguous

**Do not** write a plan `.md` file. For new streams and medium+ risk, wait for explicit human approval before implementation. If the user pushes back, iterate.

**Exit:** user has approved the plan for new streams, or the plan passes the applicable gate for non-stream low-risk work.

### 5. Execute

Write the code on the stream branch (in its worktree when one was created); never commit stream work directly to `develop` or `master`. Max ~300 lines per file. For specialist work, delegate to the appropriate skill from `repos.md`.

**After every non-trivial Write or Edit**, log the reason so the next agent understands WHY, not just what changed:

```bash
ab log-reason [<file>] "<one sentence why>"
```

Skip for: formatting, typo fixes, obvious renames. Required for: refactors, deletions, new abstractions, architectural choices.

> **⛔ Do NOT commit during Stage 5.** Code is written but never committed until Stage 6 passes and the user explicitly approves.

**When dispatching implementation sub-agents:** never include `git add` or `git commit` in the agent's task prompt. Agents write code and report back. The commit is the main agent's responsibility after Stage 6 clears and the human approves.

**After execution completes:** present a change summary inline — files modified/created, expected behavior change, what Stage 6 needs to verify.

**Backlog rule:** When you encounter a real limitation, tech debt item, or missing feature that is out of scope — do NOT fix it, do NOT open a new work stream. Append one row to `.platform/memory/BACKLOG.md` (priority / area / description / found-during / date) and continue.

**Exit:** code is written, NOT yet committed, ready for Stage 6 verification.

### 6. Verify + Gate + Learn

#### The commit gate — required before ANY `git commit`, `git push`, merge, release, or stream closure

All four must be true before committing, pushing, merging, releasing, or closing a stream:

| Gate | Requirement |
|---|---|
| ✅ Tests pass | Unit tests for every new/modified function and component |
| ✅ Security clear | Quick pass on anything touching auth, payments, or data access |
| ✅ Manual QA artifact clear | If human/app-driving verification matters, `.platform/work/qa/<stream-slug>-manual-qa.md` exists and is executable by a human tester or Maestro-style agent; otherwise the stream file records `Manual QA: not required — <specific reason>` |
| ✅ Human approves | User explicitly says "ship it" / "commit it" — the AI never self-approves |

Present Stage 6 results and the Manual QA artifact path or not-required reason to the user **before committing, pushing, merging, releasing, or closing**. Wait for the green light.

#### Testing philosophy — when and how to write tests

Not dogmatic TDD. Optimize for: **clear contracts, regression safety, fast feedback, maintainable tests, testing behavior that matters.**

**By task type:**

| Task type | Approach |
|---|---|
| 🐛 **Bug fix** | Failing test first — always. Reproduce → write failing test → fix → run suite. Proves the fix and prevents regression. |
| 🧠 **Pure business logic** | Test-first is excellent. Contract is clear, no churn. |
| ✨ **New feature (clear contract)** | Write the behavior/API test first, then implement. |
| ✨ **New feature (design moving)** | Sketch implementation first. Add tests immediately once shape stabilizes. |
| 🖼️ **UI / components** | Finish implementation first (markup/props settle), then add tests right after. |
| ♻️ **Refactor** | Lock existing behavior with characterization tests before changing internals. |
| ⚠️ **High-risk code** | Test earlier and more thoroughly. |

**The strong workflow (for new features):**
1. Define the behavior/contract.
2. Add one test for the main case.
3. Implement.
4. Add boundary / error / empty-state tests.
5. Run the nearest relevant suite.

**What NOT to do:**
- Write tests after the fact just to satisfy a coverage metric
- Test implementation details instead of behavior
- Add huge brittle tests before you understand the design
- Skip the failing regression test for a bug fix

Then verify in parallel:
- Specialist A: run tests
- Specialist B: security / code review pass (for anything security-sensitive)
- Specialist C: real-browser QA (for UI changes)

#### Manual QA artifact — required when human verification matters

At the end of Stage 6, the agent must create a durable markdown Manual QA artifact whenever the task requires human click-through, app-driving, behavior verification, bug reproduction, acceptance testing, release verification, API behavior validation, or visual review. This is a hard gate before commit, push, merge, release, or stream closure.

Default path:
```
.platform/work/qa/<stream-slug>-manual-qa.md
```

The artifact is not a planning `.md`; it is a required QA deliverable. It must be precise enough for someone who did not implement the work to execute it manually, and precise enough for a Maestro/browser/app-driving agent to translate into actions. If manual QA is not relevant, the stream file must explicitly record `Manual QA: not required — <specific reason>`. Do not delete Manual QA artifacts; when a stream closes, archive them with the stream under `.platform/work/archive/qa/`.

When the agent itself drives the app with Maestro, Browser, Playwright, MCP, or
another interactive tool, it must also create a chronological QA Execution
Journal. The Manual QA artifact says what should be tested; the execution
journal says what actually happened from the agent's perspective.

Execution journal default path:
```
.platform/work/qa/<stream-slug>-execution-journal.md
```

Execution journals are required for LLM-driven interactive QA even when every
test passes. They must record successful paths as well as failures, fixes,
human blockers, skipped checks, retests, and remaining risks. Do not delete
execution journals while the stream is open; `ab close --confirm` moves them out
of the repo with the stream (a copy is kept under `~/.agentboard/cold/`).

Use this structure:

```
## 🧪 Manual QA Artifact

🎯 Scope: <feature / bug / behavior being validated>
🧰 Environment: <local/staging/prod, URL, branch/build, browser/device, flags>
🔑 Test data: <accounts, roles, fixtures, records, permissions>
🛡️ Safety limits: <forbidden actions, rate/API caps, destructive-data rules>

✅ Happy path
1. <exact action: where to click/type/navigate> → Expected: <observable result>
2. <exact action: where to click/type/navigate> → Expected: <observable result>

🐛 Bug repro / regression
1. <original failing behavior or regression path> → Expected: <fixed behavior>

⚠️ Edge cases
- <case> → Expected: <result>
- <case> → Expected: <result>

📱 Browser/device checks: <only when relevant>
♿ Accessibility checks: <keyboard, focus, labels, contrast when relevant>
🧾 Evidence to capture: <screenshots, logs, IDs, pass/fail notes>
🤖 Maestro / automation notes: <stable selectors, flow boundaries, caps, artifacts>
✅ Signoff: <tester, date, PASS/FAIL/BLOCKED, remaining risk>
```

Execution journal structure:

```
## 🧾 QA Execution Journal

Scope:
Environment:
Driver/tooling: <Maestro MCP/CLI, Browser, Playwright, app runner, API client>
Manual QA artifact followed: <path>
Safety limits:

### Timeline
| # | Time | Tool | Action | Observation | Expected | Actual | Status | Evidence |
|---|---|---|---|---|---|---|---|---|
| 1 | <time> | <tool> | <opened app / clicked / typed / inspected / ran command> | <what the agent saw> | <expected> | <actual> | PASS/FAIL/BLOCKED/SKIPPED | <screenshot/log/report/ref> |

### Bugs, fixes, and retests
| Bug / behavior | Evidence | Diagnosis | Fix or escalation | Retest | Outcome |
|---|---|---|---|---|---|
| <issue or "None"> | <ref> | <cause or suspected layer> | <files changed / human asked / deferred> | <step rerun> | <PASS/FAIL/BLOCKED> |

### Successful paths
- <flow that passed> — evidence: <ref>

### Human requests / blockers
- <missing credential/file/decision or "None">

### Remaining risk
- <risk or "None known">
```

Rules:
- Each step starts with a concrete user action and includes an expected result.
- Include prerequisites and cleanup if test data or state must be prepared or restored.
- For bug fixes, include the original repro path and the regression check proving it stays fixed.
- For features, include at least one happy path plus the most important negative/edge path.
- Keep it concise enough to execute, but specific enough to remove guesswork.

Then **learn in three layers:**

**Layer 1 — Log (always):** append one line to `.platform/memory/log.md`:
```
YYYY-MM-DD — <task> — <outcome> — <takeaway>
```

**Layer 2 — Learnings (if bug was non-obvious):** if the root cause required >10 min to diagnose OR depended on internal behavior that isn't self-evident from the code, append an entry to `.platform/memory/learnings.md` using the L-NNN format:
```
## L-NNN — <short title>
Date: YYYY-MM-DD | Repo: <repo>
Symptom: <what the developer/user sees>
Root cause: <the actual reason>
Fix: <what was changed and where>
Class: <category — for grep>
```

**Layer 3 — Memory (if architectural):** if the insight is a stable cross-session invariant (a new pattern, a recurring gotcha, an API contract), update `memory/MEMORY.md` or a topic file under `memory/`.

**Bug investigation rule:** before diagnosing any non-obvious bug, grep `.platform/memory/learnings.md` for the symptom keyword first. Don't re-diagnose a known class of problem.

**Stream state update:** `ab checkpoint <stream-slug> --what "<what just happened>" --next "<next action>"` is the canonical way to update the stream file's `## Resume state` (current state) section — run it at the end of Stage 6, and any time you pause, switch providers, or end the session. Don't hand-edit that block; the command overwrites it atomically and trims the progress log.

**Exit:** task is done, recorded, and learned from.

---

## Stream Closure Protocol

> **Hard rule: only the human/owner declares a stream complete.**
> The AI never self-declares completion. The AI may say "I believe this stream is done — here is the evidence" and propose closure, but the final decision belongs to the developer. No exceptions.
>
> **⛔ Do NOT run steps 7–9 (archive, ACTIVE.md removal, log) until the human explicitly approves closure.** Implementation being done ≠ stream being closed. The stream file stays in `work/` and the row stays in `ACTIVE.md` until the owner says so. Steps 1–6 (verify criteria, update docs) can run after implementation. Steps 7–9 require explicit human sign-off.

Run this checklist **every time a stream reaches done** — before archiving the stream file.

> **Why:** skipping this leaves stale docs for the next session/agent. Completed features must be fully reflected in all reference files before the stream is archived.

1. **Verify done criteria** — open the stream file (`work/<slug>.md`), confirm every checkbox is ticked.
2. **Verify Manual QA artifact gate** — if human/app-driving verification matters, confirm `.platform/work/qa/<stream-slug>-manual-qa.md` exists, has exact steps plus expected results, and records pass/fail/evidence expectations. If not relevant, confirm the stream file records `Manual QA: not required — <specific reason>`. If an LLM/agent drove the app with Maestro, Browser, Playwright, MCP, or another interactive tool, also confirm `.platform/work/qa/<stream-slug>-execution-journal.md` exists and records the chronological steps, observations, bugs, fixes, retests, successful paths, evidence, and blockers.
3. **Update STATUS files** — for every repo the stream touched, mark features ✓ Done, update Last touched date, remove from Immediate priorities.
4. **Update domain file** — open `.platform/domains/<name>.md` if one exists. Update file paths, API shapes, cross-repo touch-points that changed.
5. **Deep-reference file check** — for every repo the stream touched, make an explicit YES/NO decision on whether the per-repo reference file (e.g. `backend.md`, `admin.md`) is now stale. Ask: *"Would a new developer or agent reading this file today take a wrong path?"* Update if YES. Skip if NO. This catches: new URL routes, removed fields, stack changes, patterns that no longer apply. State the decision in chat either way.
6. **Update architecture.md** — if the stream changed system topology (new endpoints, new data flows, auth changes), update the relevant section.
7. **Unblock downstream streams** — flip any `pending (blocked on this)` stream in `ACTIVE.md` to `ready-to-plan`.
8. **Close the stream and its QA artifacts** — first check: does the stream file have `closure_approved: true`? If not, **STOP**. Do not close. Ask the owner to set it. Only when `closure_approved: true` is present, write the `## Knowledge delta` in the stream file (keyed current-state facts per `### <domain> | <layer>`; see `ab close <slug>`), then run `ab close <slug> --confirm`. It merges the delta into `.platform/domains/<domain>.md`, copies the stream and its `work/qa/<slug>-manual-qa.md` / `<slug>-execution-journal.md` outside the repo (`~/.agentboard/cold/`), and removes them from `.platform/work/`. Remove the stream from `ACTIVE.md`, reset `BRIEF.md`. **Remove the closed stream from `BRIEF.md` entirely — do NOT add a "Previously completed" section.** Completed work belongs in `log.md` only. `BRIEF.md` must only ever list active streams.
9. **Log token usage** — run `ab usage log` to record the total token investment for this stream (aggregate from session reports).
10. **Append to log.md** — one line: `YYYY-MM-DD — <stream> — <outcome> — <takeaway>`.
11. **Learnings check** — any non-obvious bugs surfaced? Confirm they are in `learnings.md`. Add if missing.

**Hard rule:** steps 2–5 are not optional. If a stream touched 3 repos, all 3 STATUS files get updated and all 3 deep-reference files get an explicit YES/NO decision. The next agent should be able to open any reference file and see a correct picture of the world.

---

## Stream / Feature Analysis Protocol

> **Trigger:** When asked for a feature audit, stream analysis, "what's the state of X", "audit end to end", "check the state of", or "full analysis of \<feature/stream\>".

> ⛔ **MANDATORY — NO EXCEPTIONS:** Before doing anything else when this protocol is triggered — including collecting already-running agent results — read this section in full. A compressed context summary from a resumed session is NOT a substitute for reading this spec. The protocol steps below must be followed in order every time. Skipping any step (especially Step 3 "wait for all agents", Step 4 "exact template", Step 5 "anchor in stream file") is a protocol violation regardless of how much context you already have.

### Step 1 — Identify scope

Determine which repos are touched. If a stream file exists at `work/<slug>.md`, read it first — it defines scope, done criteria, and key files.

- Stream touches 1 repo → spawn 1 agent
- Stream touches 2 repos → spawn 2 agents
- Full platform audit → spawn 1 agent per repo

### Step 2 — Announce and dispatch in parallel

```
Dispatching N agent(s): <name1> [sonnet] — <purpose>; <name2> [sonnet] — <purpose>; ...
```

**All analysis agents run on Sonnet.** Analysis is read-only — never Opus.

Spawn one per repo (`run_in_background: true`), each covering:

| Dimension | What to check |
|---|---|
| **Implementation state** | What's built, stubbed, missing; TODO/FIXME comments |
| **Test coverage** | Which files have tests, which have zero, breadth vs. complexity |
| **Security** | Auth/permissions, hardcoded secrets, XSS, SQL injection, tenant isolation |
| **Code quality** | Files over 300 lines, business logic in views, N+1 queries, duplication |
| **Feature-specific** | *(if scoped)* Exact implementation per layer — done, missing, untested |

### Step 3 — Wait for all agents, then synthesize

Do not partially report. Wait for every agent before outputting anything.

### Step 4 — Output in standardized format

Always use this exact template:

---

```
# 📋 <Feature / Stream / Platform> — Audit Snapshot

> **Stream:** `<slug>` · **Date:** YYYY-MM-DD · **Status:** <emoji + status>
> **Repos touched:** <list>

---

## ⚡ At-a-Glance Scorecard

| | 🖥️ <Repo A> | 🎛️ <Repo B> | 📱 <Repo C> |
|---|:---:|:---:|:---:|
| **Implementation** | 🟢/🟡/🔴 | ... | ... |
| **Tests**          | 🟢/🟡/🔴 | ... | ... |
| **Security**       | 🟢/🟡/🔴 | ... | ... |
| **Code Quality**   | 🟢/🟡/🔴 | ... | ... |

> **Bottom line:** <one sentence — what is the overall state?>

---

## 🔄 How the Feature Works (End-to-End)

<ASCII flow diagram — only when it clarifies cross-layer architecture>

---

## 🛡️ Security

| Severity | Repo | Finding |
|:---:|---|---|
| 🔴 Critical / 🟡 Medium / 🟢 Clean | | |

---

## 🧪 Test Coverage

### <Repo Name>
| Area | Tested? | File |
|---|:---:|---|
| <area> | ✅ Strong / 🟡 Thin / 🔴 None | path/to/test |

---

## ✅ Implementation Status

### <Repo Name>
| Component | Status | Location |
|---|:---:|---|
| <name> | ✅ Done / ❌ Missing / ⚪ Deferred | file:line |

---

## 🔧 Open Issues

### 🔴 Must Fix (blocking)
| # | Repo | Issue |
|---|---|---|

### 🟡 Should Fix Soon
| # | Repo | Issue | Location |
|---|---|---|---|

### ⚪ Known Limitations (document, not block)
| # | Limitation |
|---|---|

---

## 🎯 Close Checklist / Priority Order

  □  1. 🧪  <test task>
  □  2. 🐛  <correctness fix>
  □  3. 🔍  <QA / verification step>
  □  4. ⚡  <performance fix>
  □  5. ✅  <closure step>
```

---

**Output rules:**
- Scorecard always first — tells the story before any detail
- Flow diagram only when it clarifies cross-layer connections
- Every security finding: 🔴 Critical / 🟡 Medium / 🟢 Clean
- Every test area: ✅ Strong / ✅ Good / 🟡 Thin / 🔴 None
- Healthy areas always listed — not just problems
- `file:line` for every actionable finding
- Tables and bullets only — no prose paragraphs

### Step 5 — Anchor the report in the stream file

> ⛔ **MANDATORY — do this before moving to Step 6 or responding about next steps.** The report is not complete until it is written into the stream file. Outputting it in chat only is a protocol violation.

Immediately after outputting the report, write it into the stream file (`work/<slug>.md`):

- **First audit on this stream:** append a new `## 🔍 Audit — YYYY-MM-DD` section at the bottom of the stream file.
- **Re-audit (audit section already exists):** replace the existing audit section entirely with the new report. Do not accumulate multiple audit sections — only the latest report stays. Add a `> Supersedes previous audit.` note at the top of the section.

```markdown
---

## 🔍 Audit — YYYY-MM-DD

> Supersedes previous audit. Run via Stream / Feature Analysis Protocol — N parallel agents.

<paste the full standardized report here>
```

This makes the report a living checklist inside the stream — every fix can be tracked against it, and any agent picking up the work in a new session has the findings immediately without re-running the analysis.

### Step 6 — Fix → Re-audit loop

Repeat until the scorecard is all 🟢:

```
┌─────────────────────────────────────────────┐
│  1. Work through 🔴 Must Fix items first     │
│  2. For each fix: implement → test → verify  │
│  3. Once all 🔴 resolved, re-run analysis    │
│     (Steps 1–4, same scope, fresh agents)    │
│  4. Compare new report to previous:          │
│     • All flagged items resolved?            │
│     • Any new issues introduced?             │
│                                              │
│  If clean → remove audit section from        │
│             stream file, proceed to closure  │
│  If not clean → return to Step 1             │
└─────────────────────────────────────────────┘
```

**Rules for the loop:**
- Never self-declare clean. Always re-run the analysis — don't assume fixes worked.
- Re-audit scope = same repos as the original audit. Don't narrow it.
- New issues found in re-audit get added to the stream file's audit section (append, don't replace).
- The audit section is removed from the stream file **only** when the re-audit returns all 🟢 with no 🔴 items remaining.

---

## Hard rules

1. **No `.md` artifacts for plans.** Plans live in chat. Only write `.md` files when they're genuinely reusable (specs, docs, conventions). **`work/` stream files, `.platform/work/qa/<stream-slug>-manual-qa.md` QA artifacts, and `.platform/work/qa/<stream-slug>-execution-journal.md` QA execution journals are exceptions — they are mandatory operational state, not plan documents. Always create required stream files (Stage 1b) and QA artifacts/journals (Stage 6) before shipping.**
2. **Max ~300 lines per file.** Extract components before hitting the limit. The agentboard dashboard flags files visually by size — treat these as mandatory signals, not suggestions:
   - **500–799 lines** (amber): growing monolith — avoid adding more code; propose extracting a module.
   - **800–999 lines** (orange): large, hard to refactor — do not add code; propose splitting before any further work.
   - **1000+ lines** (red): extreme monolith — stop, escalate to user, propose a refactor plan before ANY additions. Adding more code to a 1000-line file without splitting it first is a hard rule violation.
   When you find a file already at or near 500 lines and your task would push it further, pause, tell the user the current line count and your estimate of what the task adds, and propose a split first. The user decides whether to split now or proceed with a documented exception.
3. **Trivial tasks skip to Stage 5.** Don't bureaucratize small work.
4. **Parallelize subagents.** Never run independent subagents sequentially.
5. **Every success logs one line.** `.platform/memory/log.md` is append-only, newest-on-top.
6. **Read before you edit.** Always read the file before modifying it, even if you "know" the content.
7. **Ask before destructive actions.** Deletes, force-pushes, rollbacks, schema drops — always confirm.
8. **Never commit, push, merge, release, or close before Stage 6 + Manual QA artifact gate + human approval.** Execute produces code. Stage 6 produces tests and QA evidence. The human produces the commit/release/closure approval. No exceptions — not even for "trivial" changes.
9. **Never include `git commit` in sub-agent prompts.** Agents write code and stop. If an agent is told to commit, it bypasses tests and human approval — exactly the failure mode this rule prevents.

---

## Model profile hint

Look the model up instead of memorising a table: `ab route <task-type> [low|medium|high]` (add `--for claude` for just the alias). The data lives in `.platform/routing.tsv` (task type + complexity → tier) and `.platform/models.tsv` (tier → model per provider); edit those when models change. `ab usage report --by model` shows what was actually used.


### Workflow agent model rules (Claude Code /workflows)

**Always pass `model` explicitly in every `agent()` call.** Omitting it
inherits the caller's model — if you are running on Opus, all subagents
become Opus and token cost multiplies by agent count.

```js
// research / audit / review / test-writing → sonnet
agent("research approach", { model: "sonnet" })
agent("review the diff",   { model: "sonnet" })
agent("write unit tests",  { model: "sonnet" })

// implementation / architecture / hard decisions → opus
agent("implement feature", { model: "opus" })
agent("design the schema", { model: "opus" })

// trivial mechanical transforms → haiku
agent("rename symbol",     { model: "haiku" })
```

**All analysis agents run on Sonnet.** Analysis is read-only — never Opus.

---

## Integration with agent CLIs

- **Claude Code:** `CLAUDE.md` at project root auto-loads. Skills in `.claude/skills/` extend this workflow.
- **Codex CLI:** `AGENTS.md` at project root auto-loads. Same content as CLAUDE.md, regenerated by `scripts/sync-context.sh`.
- **Gemini CLI:** `GEMINI.md` at project root auto-loads. Same content as CLAUDE.md, regenerated by `scripts/sync-context.sh`.

All three read the same `.platform/` reference pack.
