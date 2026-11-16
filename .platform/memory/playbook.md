# Playbook

<!-- agentboard:playbook:begin -->
- **Run full checks** — `./scripts/check-rust.sh` runs formatting, strict Clippy, Rust tests, and native Swift tests.
- **Run native app** — `./scripts/dev-rust.sh` builds the SwiftUI app, embeds the Rust engine, and opens the debug bundle.
- **Run CLI scan** — `cargo run -p mac-cleaner -- scan`; mutations still require exact candidate IDs and typed confirmation.
- **Build universal package** — `./scripts/build-universal.sh` builds arm64+x86_64 CLI/app/helper plus an unsigned development DMG.
- **Native QA** — verify overview, storage history/filter/copy, cleanup plan confirmation, quarantine restore/purge confirmation, menu-bar metrics, settings, and activity without mutating live data.
- **Release** — complete signing/sandbox/notarization requirements for the chosen channel, archive in Xcode, run Intel and upgrade/rollback QA, then tag and publish.
<!-- agentboard:playbook:end -->
