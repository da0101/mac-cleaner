# Playbook

<!-- agentboard:playbook:begin -->
- **Run tests** — `python3 -m unittest`
- **Start dashboard** — `./start` then open `http://localhost:3333`
- **Run CLI scan** — `./clean`
- **Release** — run tests, commit, tag with `git tag -a vX.Y.Z`, push `main`, then push the tag.
<!-- agentboard:playbook:end -->
