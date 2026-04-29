# mac-cleaner — Decision Log

| # | Date | Topic | Decision | Why | Rejected alternatives |
|---|---|---|---|---|---|
| 1 | 2026-04-29 | Distribution | Publish as local-script open-source tool. | Tool needs local macOS access. | Hosted service. |
| 2 | 2026-04-29 | Cleanup safety | Default to preview/explicit actions for destructive cleanup. | Prevent data loss. | Fully automatic deletion. |
| 3 | 2026-04-29 | Scanner sizing | Use allocated disk usage. | Sparse VM/container files otherwise over-report. | Apparent file size only. |
| 4 | 2026-04-29 | AI provider | Keep OpenAI legacy for now; Gemini migration follow-up. | Avoid mixing provider migration into release hygiene. | Silent dual-provider fallback. |
| 5 | 2026-04-29 | Background AI | Use optional Gemini structured recommendations with local validation; no AI tool execution. | AI should help decide what to optimize without bypassing safety gates. | Letting the model directly call kill/delete tools. |
| 6 | 2026-04-29 | Chrome tabs | Close Chrome tabs only after fresh snapshot validation and explicit confirmation. | Tab indices can shift and tabs may contain unsaved work. | Autonomous bulk tab closing. |
| 7 | 2026-04-29 | AI auto optimization | `--ai` may automatically run safe RAM purge only; Gemini must never directly close apps, close tabs, kill processes, or delete files. | Owner wants automatic RAM optimization without breaking VS Code/agent jobs or destructive local state. | Fully autonomous model-driven cleanup. |
| 8 | 2026-04-29 | Dashboard widgets | Hiding AI or Chrome widgets also disables their dashboard polling timers. | Hidden panels should not keep inspecting tabs or spending AI calls. | Treating widget settings as visual-only. |
| 9 | 2026-04-29 | Server structure | Keep `server.py` as an 8-line executable shim and put dashboard state, routing, runtime, services, and assets under `mac_cleaner_server/`. | Prevent another 2k-line server monolith while preserving `./start` and `mac-cleaner`. | Replacing `http.server` or leaving dashboard code embedded in Python. |
