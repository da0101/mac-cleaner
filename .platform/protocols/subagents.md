# Protocol: Subagent model assignment (Claude Code)

> Before every `Task` / `agent()` dispatch.

Before every `Task` tool dispatch, print a manifest block in chat:

> **⚡ Dispatching N agent(s)**
>
> | Agent | Model | Task |
> |---|---|---|
> | **Name** | 🔵 `sonnet` | purpose |

Model emoji: 🟤 `fable` · 🟣 `opus` · 🔵 `sonnet` · 🟡 `haiku`. Pass `model` explicitly on every `Task` or `agent()` call — never omit it (omitting inherits the caller's model and multiplies cost by agent count).

**Pick the model with `ab route <task-type> [low|medium|high] --for claude`** (prints just the alias). The mapping lives in two editable tables — `.platform/routing.tsv` (task type + complexity → tier) and `.platform/models.tsv` (tier → model) — so there is one place to change when models are renamed. Rule of thumb it encodes: research, audit, review and test-writing run on the balanced tier; multi-file implementation and hard design on strong/frontier; renames and formatting on fast. Read-only analysis never needs the frontier tier.

**Workflow agents (`/workflows`):** same rules apply per `agent()` call inside the script — research stages → `sonnet`, implementation stages → `opus`, trivial transforms → `haiku`. Never omit `model` in a workflow — it turns a 10-agent fan-out into a 10× token multiplier.

Name agents with a persona — e.g., "Max the Mapper", "Kai the Coder". Full rule: `.platform/agents/subagent-dispatch.md`.
