# Protocol: Manual QA artifacts

> Before any commit, push, merge, release or stream closure that needs human or app-driving verification.

**Manual QA artifact rule.** Before any commit, push, merge, release, or stream closure for implementation, bug fix, debugging, feature, UI, API behavior, or release work that requires human/app-driving verification, create a durable QA markdown document at `.platform/work/qa/<stream-slug>-manual-qa.md`. Cover scope, environment, test data, safety limits, exact click/type/navigation steps, expected results, happy path, bug repro/regression steps, edge cases, browser/device checks when relevant, accessibility checks when relevant, evidence to capture, Maestro/automation notes when relevant, and pass/fail signoff. If manual QA is not relevant, record `Manual QA: not required — <specific reason>` in the stream file and explain why. Do not delete QA docs while the stream is open; `ab close --confirm` moves them out of the repo with the stream (a copy is kept under `~/.agentboard/cold/`).

**QA execution journal rule.** When an LLM/agent uses Maestro, Browser, Playwright, MCP, a simulator/emulator, or another interactive tool to drive the app, also create `.platform/work/qa/<stream-slug>-execution-journal.md`. Document every meaningful step from the agent perspective: tool used, action, observation, expected result, actual result, pass/fail/skipped status, evidence, bugs found, diagnosis, fix or human escalation, retest, successful paths, blockers, and remaining risk. Do not replace this with a final summary; the journal is the chronological trace.

> **Non-negotiable.** Skipping this means context is lost when the session ends. Applies to Claude Code, Codex CLI, and Gemini CLI equally. Trivial single-file fixes are the only exception.
