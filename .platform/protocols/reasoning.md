# Protocol: Reasoning annotation

> After every Write or Edit another agent will need to understand.

After every Write or Edit that another agent will need to understand, run:

```bash
ab log-reason [<file>] "<one sentence why>"
```

This appends a structured `Reason` event to `.platform/events.jsonl` (the file argument is optional; the active stream is resolved automatically) — full spec in `.platform/agents/commands.md` § "`ab log-reason`".

**When to call it:**
- Refactors, extractions, new abstractions
- Deletions of non-obvious code
- Architectural choices that aren't clear from the code alone
- Any change Codex or Gemini will need to understand without your conversation context

**Skip for:** formatting, typo fixes, obvious variable renames.

Examples:
```bash
ab log-reason src/auth.ts "Extracted token validation into middleware so Codex can call it from the algorithm layer without duplicating logic"
ab log-reason "Removed legacy polling loop — replaced by daemon push model"
```

This is what makes cross-provider handoff accurate. Without it, the next provider sees *what* changed but not *why*.
