# Protocol: Context handoff and delegation

> Before ending a session, switching providers, running low on context, or handing work to a specialist.

When context is running low, you're ending the session, or you're about to hand off to another CLI (Claude → Codex, Codex → Gemini, etc.), **run**:

```bash
ab checkpoint <stream-slug> --what "<1-2 lines of what just happened>" --next "<one sentence: what to do next>" \
  --cumulative-in <N> --cumulative-out <N> --provider claude --model <model-id> [--complexity trivial|normal|heavy]
```

For `--cumulative-in` / `--cumulative-out` pass Claude Code's **current session totals** (the running counters, not per-segment deltas). Agentboard subtracts what was already logged for this stream+provider+day so mid-session logging never double-counts. Omit the token fields if you don't know the counts; the checkpoint itself still works.

This overwrites the stream file's `## Resume state` block with compact "where we are" state and trims the progress log to the last 10 entries. The next agent runs `ab handoff <slug>` and picks up from there — **no re-explaining the feature**. Without this, the next agent has only stale state.

### Delegating to a specialist (multi-agent handoff)

When the user asks you to hand off work to another role (e.g. "call QA", "delegate to the security engineer", "get the refactor architect to look at this"), run:

```bash
ab delegate <role-slug> "<what the specialist should do>"
```

Example:
```bash
ab delegate qa-engineer "QA the variation carousel refactor on fix/polish-release"
```

`ab delegate` writes a handoff packet — your current BRIEF context, branch, and task — to a file that VS Code picks up and opens as a **new terminal** with Claude pre-seeded in that role.

**After running `ab delegate`, your job is done. Hard stop. Do NOT:**
- Spawn a sub-agent or use the `Agent` tool to do the delegated work
- Continue the task yourself in this session
- Interpret the command output as instructions to do more work

The specialist opens in their own terminal. You are done. Any further action on the delegated task from your session is a mistake.

### Long session fallback — `ab watch` (optional)

For heavy sessions where you may forget to checkpoint, the user can start a
background watcher:

```bash
ab watch &
```

It polls `git status` every 10 min and auto-checkpoints the active stream
whenever ≥1 tracked file has changed since the last poll. The auto-content is
mechanical (file list + carried-over Next action) — always prefer your own
checkpoints with specific `--what` / `--next`. The watcher skips ticks when
it sees a manual checkpoint happened in the last 5 minutes, so it never
clobbers fresh state. Stop with `ab watch --stop`.
