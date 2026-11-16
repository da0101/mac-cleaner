# Testing Conventions — mac-cleaner

- Run `cargo fmt --all -- --check`, workspace Clippy with warnings denied, and `cargo test --workspace`.
- Run the `MacCleanerNative` Xcode scheme tests for Swift protocol decoding and native models.
- Scanner and mutation tests use temporary directories only; never test deletion against live user data.
- Mock external owning tools and process interactions unless a read-only live QA step is explicitly called for.
- Validate the packaged `.app` contains the fixed Rust helper and that the helper reports the expected protocol version.
- `./scripts/check-rust.sh` is the combined local gate.
