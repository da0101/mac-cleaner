---
stream_id: stream-background-ai-optimizer
slug: background-ai-optimizer
type: feature
status: done
agent_owner: codex
domain_slugs: [background-ai-optimizer, ai-assistant, memory-optimization, storage-cleanup, local-dashboard]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: main
created_at: 2026-04-29
updated_at: 2026-04-29
closure_approved: true
---

# background-ai-optimizer

## Scope

- Make AI participate in the background server/dashboard optimization flow.
- Use AI to rank memory/process/storage recommendations and explain tradeoffs.
- Add hard confirmation gates before any app close, process kill, RAM purge, or file deletion.
- Prefer Gemini for the new background AI path.
- Out of scope: fully autonomous destructive cleanup without user approval.

## Done criteria

- [x] Research-backed plan approved by owner before implementation.
- [x] Dashboard/server shows AI recommendations without needing the `./ai` chat.
- [x] `mac-cleaner --ai` launcher is installable via user-local symlink.
- [x] AI recommendations are structured and safe-gated before actions.
- [x] Protected developer tools cannot be auto-closed by AI.
- [x] Tests cover recommendation parsing/gating behavior.
- [x] `.platform/memory/log.md` appended.
- [x] `decisions.md` updated if any architectural choices were made.

## Key decisions

_Append-only. Format: `YYYY-MM-DD — <decision> — <rationale>`_

2026-04-29 — Background AI is optional via `./start --ai` — keeps default local/rules-only behavior and avoids surprise API calls.
2026-04-29 — Gemini returns structured recommendations only — local code validates and gates all actions.
2026-04-29 — `--ai` may auto-run safe RAM purge only — app/tab/process closing still requires a separate confirmation design.

## Resume state
_Overwritten by `ab checkpoint` — the compact payload the next agent reads first. Keep this block under ~10 lines._

- **Last updated:** 2026-04-29 by danilulmashev
- **What just happened:** Fixed audit cleanup: AI widget hiding stops dashboard polling, docs/status now describe --ai safe RAM purge only, browser QA passed, stream moved to closure-review.
- **Current focus:** —
- **Next action:** Ask owner for explicit closure approval; if approved set closure_approved true and run stream closure protocol.
- **Blockers:** none

## Progress log

2026-04-29 18:43 — Fixed audit cleanup: AI widget hiding stops dashboard polling, docs/status now describe --ai safe RAM purge only, browser QA passed, stream moved to closure-review.

2026-04-29 18:23 — Audited stream and anchored report in stream file; fixed dashboard header wrapping and noted remaining polling/docs/status cleanup items.

2026-04-29 15:58 — Changed --ai to automatic safe RAM optimization, verified mac-cleaner --help and safe purge path, and captured Chrome tab advisor as follow-up.

2026-04-29 15:52 — Added user-local install/uninstall scripts and fixed start to resolve symlinked mac-cleaner command; verified mac-cleaner --help.

2026-04-29 15:45 — Implemented optional Gemini dashboard advisor, safety validator, UI panel, startup --ai/--no-ai modes, docs, and tests.

2026-04-29 15:28 — Registered stream/domain and completed initial local plus Gemini docs research.
2026-04-29 15:42 — Implemented optional background AI advisor and validation tests.

2026-04-29 15:29 — Registered background AI optimizer stream.

## Open questions

- Should the first implementation run AI continuously on a timer, or only when memory/storage thresholds are crossed?
- Should AI recommendations appear only in the dashboard, or also in terminal logs?
- Chrome tabs can be inspected through AppleScript, but closing tabs needs a dedicated confirmation UX and privacy rules for URLs/titles.

---

## 🔍 Audit — 2026-04-29

> Supersedes previous placeholder. Run via Stream / Feature Analysis Protocol — local read-only audit; subagent dispatch skipped because this Codex session requires explicit user approval for delegated agents.

# 📋 Background AI Optimizer — Audit Snapshot

> **Stream:** `background-ai-optimizer` · **Date:** 2026-04-29 · **Status:** 🟢 closure-review
> **Repos touched:** mac-cleaner

---

## ⚡ At-a-Glance Scorecard

| | 🖥️ mac-cleaner |
|---|:---:|
| **Implementation** | 🟢 |
| **Tests**          | 🟢 |
| **Security**       | 🟢 |
| **Code Quality**   | 🟡 |

> **Bottom line:** The AI optimizer is implemented, safety-gated, browser-smoke-tested, and ready for owner closure review; remaining server-size debt is tracked as non-blocking.

---

## 🔄 How the Feature Works (End-to-End)

```text
mac-cleaner --ai
  -> server.py parses AI mode
  -> server.py builds RAM/storage/process snapshot
  -> ai_advisor.py asks Gemini or local fallback
  -> ai_advisor.py validates recommendations locally
  -> dashboard displays recommendations
  -> only safe RAM purge may run automatically; app close/delete remain confirmation-gated
```

---

## 🛡️ Security

| Severity | Repo | Finding |
|:---:|---|---|
| 🟢 Clean | mac-cleaner | Gemini output is validated locally; protected processes are downgraded to review/danger and never executable. `ai_advisor.py:378` |
| 🟢 Clean | mac-cleaner | Storage recommendations use scanner item names and safety tiers, not arbitrary paths. `ai_advisor.py:444` |
| 🟢 Clean | mac-cleaner | `--ai` only auto-runs RAM purge; app/tab/process closing and storage deletion are not executed directly by AI. `server.py:645` |

---

## 🧪 Test Coverage

### mac-cleaner
| Area | Tested? | File |
|---|:---:|---|
| Protected process downgrade | ✅ Strong | `tests/test_ai_advisor.py:42` |
| Report-only storage downgrade | ✅ Strong | `tests/test_ai_advisor.py:63` |
| Closeable process still non-executable | ✅ Strong | `tests/test_ai_advisor.py:83` |
| Settings sanitization/persistence | ✅ Good | `tests/test_settings.py:8` |
| Dashboard AI panel/browser behavior | ✅ Good | Playwright smoke `/tmp/playwright-test-mac-cleaner-dashboard.js` |
| End-to-end local HTTP API behavior | ✅ Good | `curl /` returned `200` with dashboard HTML |

---

## ✅ Implementation Status

### mac-cleaner
| Component | Status | Location |
|---|:---:|---|
| CLI AI modes `--ai`, `--ai-advisory`, `--no-ai` | ✅ Done | `server.py:83` |
| Structured AI snapshot | ✅ Done | `server.py:500` |
| Gemini/local recommendation engine | ✅ Done | `ai_advisor.py:213` |
| Local safety validator | ✅ Done | `ai_advisor.py:378` |
| Automatic safe RAM optimization | ✅ Done | `server.py:645` |
| Settings JSON/schema | ✅ Done | `server.py:92`, `settings.schema.json` |
| Dashboard settings modal and theme button | ✅ Done | `server.py:1083` |

---

## 🔧 Open Issues

### 🔴 Must Fix (blocking)
| # | Repo | Issue |
|---|---|---|
| 1 | mac-cleaner | None found in the current audit. |

### 🟡 Should Fix Soon
| # | Repo | Issue | Location |
|---|---|---|---|
| 1 | mac-cleaner | The dashboard UI is embedded in a large `server.py`; browser smoke now covers the risky interactions, but extraction remains follow-up debt. | `server.py:970` |

### ⚪ Known Limitations (document, not block)
| # | Limitation |
|---|---|
| 1 | `purge` effectiveness depends on macOS behavior and sudo/NOPASSWD availability. |
| 2 | AI cost is controlled by interval settings, but no token/call counter is displayed yet. |
| 3 | The legacy `./ai` assistant still uses OpenAI separately from the Gemini background advisor. |

---

## 🎯 Close Checklist / Priority Order

  ☑  1. 🧪 Add lightweight HTTP/dashboard smoke coverage for `/`, settings modal, and widget behavior.
  ☑  2. 🐛 Gate AI polling on `show_ai_recommendations`.
  ☑  3. 🔍 Align root docs and stream status with implemented `--ai` behavior.
  ☑  4. ⚡ Add focused browser render/interaction QA before further UI growth.
  □  5. ✅ Ask owner for explicit closure approval; do not archive without `closure_approved: true`.
