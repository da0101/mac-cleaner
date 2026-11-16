# Rust Conventions — mac-cleaner

- Rust is authoritative for discovery, policy, plans, filesystem mutation, persistence, system metrics, and protocol responses.
- Keep `unsafe_code = "deny"`; prefer the one-shot JSON process boundary over FFI.
- Commands accept compiled catalog IDs, UUIDs, and bounded values—not paths supplied by a UI.
- Avoid `unwrap` and `expect`; errors cross layers as typed failures and protocol-safe messages.
- Scanner work is bounded, mount-aware, symlink-safe, allocated-size based, and tested in temporary homes.
- Run formatting, strict workspace Clippy, and all workspace tests before handoff.
