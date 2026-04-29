# Playbook

<!-- agentboard:playbook:begin -->
- **Run tests** — `python3 -m unittest`
- **Start dashboard** — `./start` then open `http://localhost:3333`
- **Run CLI scan** — `./clean`
- **Dashboard QA** — use Playwright against `http://127.0.0.1:3333` with installed Google Chrome; verify settings modal, theme toggle, and hidden-widget polling before closing UI streams.
- **Release** — run tests, commit, tag with `git tag -a vX.Y.Z`, push `main`, then push the tag.
<!-- agentboard:playbook:end -->
