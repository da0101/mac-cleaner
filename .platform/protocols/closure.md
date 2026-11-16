# Protocol: Stream closure

> When a stream's work is done and the human has approved closing it.

## Only the human closes a stream

**Only the human/owner declares a stream complete.** You never self-declare completion. You may say "I believe this is done — here is the evidence" and propose closure, but the final decision belongs to the developer. No exceptions.

Before closing you MUST:
1. Present evidence of completion to the user.
2. Wait for explicit human sign-off.
3. Verify the stream file has `closure_approved: true`.

**Enforcement:** Claude Code enforces this mechanically via `platform-closure-gate.js` (PreToolUse hook — blocks any edit to `ACTIVE.md` that removes a stream row without `closure_approved: true`). Codex CLI and Gemini CLI have no hook system — follow this through instruction discipline. The absence of a mechanical gate does not make it optional.

## The two-step close

```bash
ab close <stream-slug>              # step 1 — print the harvest checklist
ab close <stream-slug> --confirm    # step 2 — merge knowledge, remove the stream
```

**Step 1 — harvest.** Following the checklist, write what survives the stream:

- **`## Knowledge delta` in the stream file** — durable facts about the system, merged into `.platform/domains/<domain>.md` on `--confirm`:

  ```
  ## Knowledge delta
  ### <domain-slug> | <layer>
  - [stable-key] current-state fact, with `file/paths` in backticks
  - [old-key] -                      removes a fact that is no longer true
  ```

  State the **current truth**, not what changed. Same key replaces, new key adds. The layer must appear in an existing `## heading` of the card (run `ab recall <domain>` to see them) or a new section is created.
- `.platform/memory/gotchas.md` — landmines discovered (🔴 never-forget · 🟡 usually-matters · 🟢 minor)
- `.platform/memory/playbook.md` — shortcuts / rituals / commands worth recording
- `.platform/memory/open-questions.md` — anything still unresolved (or move a prior question to Resolved)
- `.platform/memory/decisions.md` — locked architectural/product/tooling decisions
- `.platform/memory/learnings.md` — non-obvious bug root-causes

Skip categories with nothing to add.

**Step 2 — close.** `--confirm` (needs `closure_approved: true`) merges the delta into the domain cards, records a summary line in `knowledge/streams.jsonl`, copies the stream and its QA files to `~/.agentboard/cold/<project>/closed/`, removes them from `.platform/work/`, and updates `ACTIVE.md` and `log.md`. Use `--dry-run` to preview the merge.

Then reset `BRIEF.md` (remove the closed stream; do not add a "previously completed" section — completed work belongs in `log.md`).

Full protocol: `.platform/workflow.md` → "Stream Closure Protocol".
