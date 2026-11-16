# Protocol: New task bootstrap

> Before any non-trivial task that is not already tracked in `work/ACTIVE.md`.

When the user gives a task not already tracked in `ACTIVE.md`, **stop and complete these steps before any research, planning, or code**:

1. **Check `ACTIVE.md`** — does this stream already exist? If yes, load it and continue.
2. **Check `.platform/domains/`** — does a domain file exist for this feature area?
   - No → **create `.platform/domains/<name>.md`** with cross-layer touch-point inventory first.
3. **Create `work/<stream-slug>.md`** — what we're building, why, done criteria, key files, current state.
4. **Add a row to `work/ACTIVE.md`** — slug / type / in-progress / agent / date.
5. **Update `work/BRIEF.md`** — point to new primary stream + list domain file under "Relevant context".
6. Only then proceed to triage → research → propose → execute.

**Research-first new stream rule.** If the request is a new stream, research is always required before planning or implementation. Scale the depth to risk, but include local context and targeted external research every time. Present the research-backed phased plan, risks, mitigations, alternatives, tests, rollback path, and open questions; wait for human validation/approval before implementation. If implementation diverges from the approved plan, pause and ask.

**Worktree branch rule.** Work on a `feature/<slug>` or `bugfix/<slug>` branch (from `develop`; `hotfix/<slug>` from `master` only when the user explicitly says hotfix) in the repo's main checkout. Do not create a worktree by default — worktrees are on demand: only when the user asks, when two sessions must edit the same repo at once, or when the main checkout holds unrelated uncommitted work. Create one with `ab worktree new <slug>` and record it in the stream file; `ab worktree status` shows leftovers and `ab worktree clean --apply` removes merged ones. Install the repo's development dependencies in the checkout you work from, identify the local dev command and localhost port(s), and record them in the stream file before coding or QA.
