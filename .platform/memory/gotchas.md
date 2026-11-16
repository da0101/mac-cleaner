# Gotchas

<!-- agentboard:gotchas:begin -->
- 🔴 [repo] — Never commit `.env`, credentials, runtime SQLite data, cleanup history, caches, or local AI workspace folders.
- 🔴 [storage-cleanup] — System Data is not a folder to delete; only delete classified child paths.
- 🟡 [memory-optimization] — Raw free RAM is misleading on macOS; use available/reusable memory.
- 🟡 [native-engine] — Never let Swift supply cleanup paths or shell fragments; resolve and validate the fixed `Contents/Helpers/mac-cleaner-engine` bundle path, then pass only fixed commands plus opaque IDs. `Bundle.url(forAuxiliaryExecutable:)` did not discover the packaged helper in live QA.
- 🟢 [agentboard] — The post-commit hook may update stream/log files after a commit; check `git status` before tagging.
<!-- agentboard:gotchas:end -->
