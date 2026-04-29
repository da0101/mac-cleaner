# mac-cleaner — Decision Log

| # | Date | Topic | Decision | Why | Rejected alternatives |
|---|---|---|---|---|---|
| 1 | 2026-04-29 | Distribution | Publish as local-script open-source tool. | Tool needs local macOS access. | Hosted service. |
| 2 | 2026-04-29 | Cleanup safety | Default to preview/explicit actions for destructive cleanup. | Prevent data loss. | Fully automatic deletion. |
| 3 | 2026-04-29 | Scanner sizing | Use allocated disk usage. | Sparse VM/container files otherwise over-report. | Apparent file size only. |
| 4 | 2026-04-29 | AI provider | Keep OpenAI legacy for now; Gemini migration follow-up. | Avoid mixing provider migration into release hygiene. | Silent dual-provider fallback. |

