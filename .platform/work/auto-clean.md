# auto-clean

Type: feature · Status: in-progress · Agent: claude · Started: 2026-11-17

## Why
Owner: the app must keep the Mac clean on its own, especially RAM. macOS cannot "clean" RAM
without ending work (`purge` only drops disk cache), so the real lever is quitting idle apps.

## Approved scope (owner "Yes, build it", 2026-11-17)
1. RAM relief: when conservative available memory stays below a threshold for a set time, ask the
   largest app from the owner's opt-in list to quit normally. Developer tools, Docker, Finder and
   system apps can never be listed.
2. Scheduled storage cleanup of opted-in catalog categories only, idle 24+ hours, through the
   existing plan -> quarantine -> purge pipeline.
3. Everything off by default; every action logged; owner can pause by switching it off.
Policy change recorded in CLAUDE.md / AGENTS.md / GEMINI.md constraints 3, 4 and 7.

## Design
- `cleaner-domain/auto.rs`: settings, decisions, log types.
- `cleaner-policy/auto.rs`: pure `plan_ram_relief`, `validate_auto_settings`, `app_bundle`, never-quit list.
- `cleaner-service/auto.rs`: `auto_tick`, `auto_report_quit`, scheduled `run_auto_storage`.
- Engine commands: auto-settings, auto-settings-set, auto-tick, auto-report-quit, auto-log.
- Swift: 30 s loop in `AppModel`, `AutoCleanView`. Swift only calls `NSRunningApplication.terminate()`
  for the bundle path Rust chose (inside an Applications folder). Rust never signals a process.

## Knowledge delta
- `top_processes` returns only the top 20; auto-clean needs `SystemMonitor::all_processes` to total helpers.
- An app with any protected helper process is skipped as a whole.
- An app asked to quit twice within 30 minutes is left alone (it refused).
