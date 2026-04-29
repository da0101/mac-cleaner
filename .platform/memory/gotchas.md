# Gotchas

<!-- agentboard:gotchas:begin -->
- 🔴 [repo] — Never commit `.env`, `.venv`, cleanup history, or local AI workspace folders.
- 🔴 [storage-cleanup] — System Data is not a folder to delete; only delete classified child paths.
- 🟡 [memory-optimization] — Raw free RAM is misleading on macOS; use available/reusable memory.
- 🟢 [agentboard] — The post-commit hook may update stream/log files after a commit; check `git status` before tagging.
<!-- agentboard:gotchas:end -->
