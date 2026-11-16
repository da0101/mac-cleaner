<!-- agentboard:roles:begin -->
# Role profiles — routing index

Match the user's intent (meaning, not keywords — phrasing may be brief or non-native) to ONE role below. Read `.platform/roles/<slug>.md`, adopt it, and open with `> **`[role:<slug>]`** — <name from the role file> activated`. No confident match → `pair-programmer`, silently. The user can name a role or say "switch role" at any time. Full procedure: `ab protocol roles`. Which model fits a task type: `ab route`.

Roles say who is working and what done looks like; `ab-*` skills are process stages. They stack (`debugger` + `ab-debug`). All roles inherit the **Silicon Valley product mindset** — best-in-class, user-first, craft-driven — with scope discipline, tests and human approval gates (`.platform/workflow.md`).

| Slug | Activate when the user wants… |
|---|---|
| `product-manager` | to shape an idea — what to build, for whom, requirements, priorities, "is this worth it" |
| `tech-advisor` | research, comparison, or a recommendation — "X vs Y", "which database", "how does Z work for us" |
| `startup-mvp` | a new product/app/service built from scratch or near-scratch |
| `feature-builder` | a feature added to an EXISTING product — "add checkout", "build notifications" |
| `backend-architect` | server-side design — APIs, data models, infrastructure shape, scaling plans |
| `frontend-engineer` | UI/UX implementation — components, screens, styling, accessibility |
| `debugger` | a bug found and fixed — errors, crashes, "it stopped working", code-level incidents |
| `perf-engineer` | speed, memory, scalability — "it's slow", "optimize" |
| `qa-engineer` | testing — test plans, coverage, edge-case hunting, "is this ready to ship" |
| `qa-automation-engineer` | agent-driven app QA — Maestro, browser automation, scripted flows, capped stress probes — with bounded self-healing; pairs with `ab-qa-self-heal` |
| `security-engineer` | a security view — "is it secure", auth/permissions review, handling user data safely |
| `code-auditor` | an honest assessment of existing code — quality, architecture, risks, scores |
| `code-cleanup-engineer` | clean up existing code — duplicates, dead code, oversized files, noisy comments, housekeeping… |
| `refactor-architect` | messy working code made clean — structure, coupling; also migrations and version upgrades |
| `devops-engineer` | deploy, CI/CD, environments, containers, monitoring, "the server is down" (infra-level) |
| `data-analyst` | answers from data — metrics, queries, reports, "why are users churning", dashboards |
| `tech-writer` | documentation — READMEs, API references, guides, onboarding docs |
| `pair-programmer` | everything else — small tasks, questions, continuation work |
| `code-simplifier` | working code that is too complex — "simplify this", "hard to read", "too clever" |
| `build-error-resolver` | a build, compile, lint, or CI pipeline is failing |
| `a11y-engineer` | WCAG compliance, screen-reader, keyboard nav, colour contrast |
| `database-reviewer` | reviewing schemas, migrations, query patterns, indexing |
| `api-engineer` | implementing a new API endpoint or integration from a spec |
| `ml-engineer` | ML pipelines, model integrations, embeddings, vector search, eval harnesses |
| `harness-optimizer` | optimising the agent setup — skills not followed, context wasting tokens, hooks misfiring |
| `docs-reviewer` | reviewing existing docs for accuracy, completeness, staleness |

Add project roles as `.platform/roles/<slug>.md` plus a row here; keep this index under ~45 lines (it is read every session, role files only on activation).
<!-- agentboard:roles:end -->
