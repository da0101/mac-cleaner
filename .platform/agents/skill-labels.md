# Skill Labels — Visual Identity System

Every ab skill announces itself at the start of each response with a labeled blockquote:

> **`[ab-research]`**

This makes it immediately clear which skill is active during multi-skill workflows.

## Label map

| Skill | Label | ANSI color | Role |
|-------|-------|-----------|------|
| ab-triage | `[ab-triage]` | 220 (gold) | Classify task type / scope / risk |
| ab-workflow | `[ab-workflow]` | 39 (blue) | Orchestrate the 6-stage workflow |
| ab-research | `[ab-research]` | 117 (periwinkle) | Bounded research before planning |
| ab-pm | `[ab-pm]` | 214 (orange) | Product thinking / user value framing |
| ab-architect | `[ab-architect]` | 141 (purple) | System / component design |
| ab-test-writer | `[ab-test-writer]` | 120 (green) | Unit test generation |
| ab-security | `[ab-security]` | 196 (red) | Security audit |
| ab-qa | `[ab-qa]` | 226 (yellow) | Manual / browser QA |
| ab-qa-self-heal | `[ab-qa-self-heal]` | 202 (orange) | Agent-driven QA self-heal |
| ab-review | `[ab-review]` | 183 (lavender) | Pre-PR code review |
| ab-debug | `[ab-debug]` | 208 (amber) | Root-cause bug investigation |

## Role labels

Role profiles (`.platform/roles/`) use the same convention with `[role:<slug>]` labels — same blockquote format, same optional ANSI color on raw terminals (each role file declares its color). Roles and ab-* skill labels **stack**: the role says *who is working and what done looks like*; the skill says *which process stage is running*. When both are active, the role label comes first:

> **`[role:debugger]`** **`[ab-debug]`**

## Rendering

- **Claude Code / markdown UIs**: the blockquote renders with a colored left border; bold code renders with distinct background. No ANSI needed.
- **Raw terminal CLIs (Gemini, Codex)**: skills may additionally output the ANSI escape sequence for true color — `\033[38;5;COLORm[ab-X]\033[0m`.

## Rule

**Every skill response MUST start with the identity blockquote.** No exceptions. This is how the user tracks which agent is speaking in multi-skill sessions.
