# Testing Conventions — mac-cleaner

- Use `python3 -m unittest`.
- Scanner tests must use temp directories only.
- Mock subprocesses for future process/RAM tests.
- Never test real deletion outside temp fixtures.
- Run `python3 -m py_compile scanner.py cleaner.py server.py ai_cleaner.py ai_advisor.py browser_tabs.py mac_cleaner_server/*.py` before release.
