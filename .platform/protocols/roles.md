# Protocol: Role activation

> First substantive task of a session, and whenever the task type clearly shifts.

On the **first substantive task message** — and again whenever the task type clearly shifts — before starting the work:

1. Consult `.platform/roles/INDEX.md` and match the user's **intent** against its routing table. Match meaning, not keywords — phrasing may be brief or non-native English ("make me app for gym").
2. Read the single matching `.platform/roles/<slug>.md` and adopt it: identity, process, deliverables, constraints.
3. Announce it — start your response with the role label blockquote (`> **`[role:<slug>]`** — <Role name> activated`) before any work.
4. No confident match → work as `pair-programmer` silently (no announcement).

**Silicon Valley product mindset:** PM and engineering roles must think like
best-in-class Silicon Valley product teams: user-obsessed, future-facing,
innovative, craft-driven, fast, and rigorous. Raise the bar beyond basic task
completion, but never use this as permission for vague hype, scope creep,
unapproved features, weak tests, or skipping approval gates. See
`.platform/workflow.md` for the full rule.

**Manual override always wins** — if the user names a role or says "switch role", use that role. `ab role list` shows the available roles.

**Superpowers integration:** Role activation runs **before** any `superpowers:*` skill invocation. This is an explicit override of the `using-superpowers` "invoke skills before any response" rule — CLAUDE.md instructions are the highest priority tier in the superpowers hierarchy, and this sentence is the assertion. Once the role is active, superpowers skills serve as process tools *within* it: `feature-builder` + `superpowers:brainstorming` means you are a Senior Product Engineer using the brainstorming process — not a generic agent. Announce the role first, then invoke the skill.

**Rescan trigger** — when the user says "update the platform", "rescan", "refresh context", "you're out of date", or similar: run `ab rescan`, then read `.platform/RESCAN.md` and follow the update protocol.
