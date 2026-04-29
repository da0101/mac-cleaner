---
stream_id: stream-browser-tab-optimizer
slug: browser-tab-optimizer
type: feature
status: done
agent_owner: codex
domain_slugs: [browser-tab-optimizer, background-ai-optimizer, memory-optimization, local-dashboard]
repo_ids: [mac-cleaner]
base_branch: main
git_branch: main
created_at: 2026-04-29
updated_at: 2026-04-29
closure_approved: true
---

# browser-tab-optimizer

## Scope

- Add Chrome tab inventory and memory-optimization recommendations.
- Let Gemini/local rules rank likely useless, duplicate, or stale-looking tabs.
- Add dashboard controls to review and close selected recommended tabs.
- Keep developer/agent safety: do not close apps, windows, or all tabs automatically.
- Out of scope: fully autonomous tab closing without confirmation.

## Done criteria

- [x] Research-backed plan approved by owner before implementation.
- [x] Chrome tab snapshot works without exposing arbitrary commands.
- [x] Dashboard shows tab recommendations and close controls.
- [x] Tab close endpoint validates against a fresh snapshot.
- [x] Tests cover tab parsing/recommendation validation.
- [x] `.platform/memory/log.md` appended.
- [x] `decisions.md` updated if architectural choices were made.

## Key decisions

_Append-only. Format: `YYYY-MM-DD — <decision> — <rationale>`_

2026-04-29 — Close one confirmed tab at a time — Chrome tab indices shift after close operations.
2026-04-29 — Strip URL query/fragment from tab snapshots — reduce accidental private data exposure.

## Resume state
_Overwritten by `ab checkpoint` — the compact payload the next agent reads first. Keep this block under ~10 lines._

- **Last updated:** 2026-04-29 by danilulmashev
- **What just happened:** Fixed audit cleanup: hidden Chrome widget stops polling, tab close fallback validates title/domain/url, converter detection remains covered, browser QA passed, stream moved to closure-review.
- **Current focus:** —
- **Next action:** Ask owner for explicit closure approval; if approved set closure_approved true and run stream closure protocol.
- **Blockers:** none

## Progress log

2026-04-29 18:43 — Fixed audit cleanup: hidden Chrome widget stops polling, tab close fallback validates title/domain/url, converter detection remains covered, browser QA passed, stream moved to closure-review.

2026-04-29 18:23 — Audited stream and anchored report; expanded Chrome tab detection to catch converter/downloader tabs and let Gemini consider disposable non-active tabs beyond seed candidates.

2026-04-29 18:16 — Moved dashboard settings into a modal and added a top-bar light/dark toggle; settings still persist to ignored settings.json.

2026-04-29 18:09 — Added persisted settings JSON/schema and dashboard controls for AI/Chrome intervals, auto-clean/RAM cadence, target RAM, widget visibility, and light/dark/system theme.

2026-04-29 17:40 — Finished Chrome tab optimizer and verified tests, syntax, installed command help, and local Chrome tab endpoints.

2026-04-29 17:39 — Implemented Chrome tab snapshot/recommendations, Gemini/local tab ranking, dashboard panel, confirmed single-tab close endpoint, docs, and tests.

2026-04-29 16:00 — Registered Chrome tab optimizer stream.
2026-04-29 16:05 — Implemented Chrome tab optimizer APIs, dashboard panel, and tests.

## Open questions

- Should tab recommendations include full URLs, domains only, or both?
- Should “close selected tabs” be the first version, or also a one-click “close all recommended tabs” with confirmation?

---

## 🔍 Audit — 2026-04-29

> Supersedes previous placeholder. Run via Stream / Feature Analysis Protocol — local read-only audit; subagent dispatch skipped because this Codex session requires explicit user approval for delegated agents.

# 📋 Browser Tab Optimizer — Audit Snapshot

> **Stream:** `browser-tab-optimizer` · **Date:** 2026-04-29 · **Status:** 🟢 closure-review
> **Repos touched:** mac-cleaner

---

## ⚡ At-a-Glance Scorecard

| | 🖥️ mac-cleaner |
|---|:---:|
| **Implementation** | 🟢 |
| **Tests**          | 🟢 |
| **Security**       | 🟢 |
| **Code Quality**   | 🟡 |

> **Bottom line:** Chrome tab inspection, recommendation, and confirmed single-tab close are implemented, browser-smoke-tested, and ready for owner closure review; remaining server-size debt is tracked as non-blocking.

---

## 🔄 How the Feature Works (End-to-End)

```text
dashboard
  -> /api/chrome-tab-recommendations
  -> server.py refresh_chrome_tab_recommendations()
  -> browser_tabs.py reads Chrome via JXA
  -> local rules seed duplicate/blank/converter/media candidates
  -> ai_advisor.py optionally asks Gemini to rank non-active disposable tabs
  -> dashboard shows review-only close controls
  -> /api/chrome-tabs/close validates fresh snapshot and closes one confirmed tab
```

---

## 🛡️ Security

| Severity | Repo | Finding |
|:---:|---|---|
| 🟢 Clean | mac-cleaner | Browser inspection is fixed to Google Chrome JXA; the browser never sends arbitrary shell commands. `browser_tabs.py:60` |
| 🟢 Clean | mac-cleaner | URLs are normalized to remove query/fragment before recommendation display. `browser_tabs.py:97` |
| 🟢 Clean | mac-cleaner | Active tabs are rejected during recommendation validation and again before close. `ai_advisor.py:350`, `server.py:607` |
| 🟢 Clean | mac-cleaner | Closing is one confirmed tab at a time through a local endpoint bound to the dashboard server. `server.py:1913` |

---

## 🧪 Test Coverage

### mac-cleaner
| Area | Tested? | File |
|---|:---:|---|
| URL privacy normalization | ✅ Strong | `tests/test_browser_tabs.py:52` |
| Duplicate and blank tab candidates | ✅ Strong | `tests/test_browser_tabs.py:58` |
| Converter/downloader tab candidates | ✅ Good | `tests/test_browser_tabs.py:67` |
| AI tab validation rejects active/unknown tabs | ✅ Strong | `tests/test_browser_tabs.py:74` |
| Fresh close matching prefers tab ID | ✅ Good | `tests/test_browser_tabs.py:87` |
| Dashboard Chrome tab UI and modal interactions | ✅ Good | Playwright smoke `/tmp/playwright-test-mac-cleaner-dashboard.js` |
| Real Chrome close flow | 🟡 Thin | close remains confirmation-gated; matching logic covered in unit tests |

---

## ✅ Implementation Status

### mac-cleaner
| Component | Status | Location |
|---|:---:|---|
| Chrome tab snapshot via JXA | ✅ Done | `browser_tabs.py:18` |
| Query/fragment stripping | ✅ Done | `browser_tabs.py:97` |
| Duplicate/blank/media local rules | ✅ Done | `browser_tabs.py:121` |
| Converter/downloader local rule | ✅ Done | `browser_tabs.py:146` |
| Gemini tab ranking beyond seed candidates | ✅ Done | `ai_advisor.py:285` |
| Chrome recommendations endpoint | ✅ Done | `server.py:1875` |
| Confirmed close endpoint | ✅ Done | `server.py:1913` |
| Dashboard top-row theme/settings controls | ✅ Done | `server.py:1083` |
| Hidden-widget polling gate | ✅ Done | `server.py:1282`, `server.py:1791` |
| Strict close fallback validation | ✅ Done | `browser_tabs.py:208` |

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
| 1 | Chrome does not expose reliable per-tab last-visited age through this JXA approach. |
| 2 | Gemini only receives tab title/domain/active/index metadata, not full private URLs. |
| 3 | Converter/downloader detection is heuristic and should be tuned from real missed examples. |

---

## 🎯 Close Checklist / Priority Order

  ☑  1. 🧪 Add browser/UI smoke coverage for the dashboard, settings modal, and Chrome panel.
  ☑  2. 🐛 Gate Chrome polling on `show_chrome_tab_optimizer`.
  ☑  3. 🔍 Strengthen close validation by matching title/domain/url when `tab_id` is missing.
  ☑  4. ⚡ Add focused browser render/interaction QA before more UI changes.
  □  5. ✅ Ask owner for explicit closure approval; do not archive without `closure_approved: true`.
