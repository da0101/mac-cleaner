# Full Rust rewrite interactive execution journal

This journal records native-computer QA interactions and their intent. No destructive action was authorized through the UI.

| Sequence | Interaction | Reason | Outcome |
|---|---|---|---|
| 1 | Started `./scripts/dev-rust.sh` | Verify the real Tauri lifecycle and bundled frontend against the Rust backend. | First build completed; native process opened. |
| 2 | Requested native app state for `Mac Cleaner` | Read the accessibility tree and screenshot before any interaction. | Timed out with Apple Event error `-1712`; no input occurred. |
| 3 | Requested the running-app list | Determine whether the accessibility failure was app-name selection or host permission. | Timed out with the same error; no input occurred. |
| 4 | Captured a read-only local screen image | Verify launch/rendering without driving the UI after accessibility failed. | Confirmed native rendering and live metrics. Screenshot retained only in `/tmp`. |
| 5 | Stopped the developer app with `Ctrl-C` in its launch terminal | End the QA process cleanly before correcting startup state. | Tauri/Vite developer session exited normally. |
| 6 | Moved exact file `~/Library/LaunchAgents/Mac Cleaner.plist` to `/tmp/Mac Cleaner.plist.qa-created-20260810` | Reverse the Rust debug login item observed during QA without deleting it. | Source no longer exists; backup remains recoverable in `/tmp`; legacy `com.mac-cleaner.menubar.plist` unchanged. |
| 7 | Relaunched the patched Tauri developer app and captured a read-only local screenshot | Verify the startup gate and updated interface. | Login toggle was disabled with `Installed app only`; no Rust LaunchAgent was created. |
| 8 | Ran Rust CLI `status --json` with aggregate filtering | Cross-check the native Rust monitor without exposing full process/path details. | Returned valid memory/disk/process aggregates. |
| 9 | Ran Rust CLI `scan --json` with aggregate filtering | Verify scanner startup and real-home read-only behavior without cleanup. | Zero candidates, one protected/unsafe skip, no mutation. |
| 10 | Stopped the developer app with `Ctrl-C` | Leave no QA server or native process running. | Developer session exited normally. |
| 11 | Removed only the isolated worktree's generated Cargo `target/` directory | Recover 4.0 GiB so both release architectures could build without filling the host volume. | Source and user data were untouched; 6.2 GiB became available. |
| 12 | Ran `./scripts/build-universal.sh` | Verify real arm64+x86_64 CLI/app packaging. | Universal CLI, `.app`, and DMG built; both binaries contain arm64 and x86_64 slices. |
| 13 | Generated and integrated the application icon | Replace the placeholder mark with a production shield/sweep/sparkle identity. | Source PNG, ICNS/PNG icon set, packaged bundle icon, and in-app header icon verified. |
| 14 | Launched the packaged universal `.app` directly | Verify the actual bundle, installed-app login gate, live monitor, and final logo without installing it. | App rendered correctly; login remained off; no LaunchAgent created. |
| 15 | Stopped the packaged app with `Ctrl-C` | Leave no QA process running. | Process exited; no startup state changed. |
| 16 | Copied final universal outputs to `artifacts/full-rust-rewrite-alpha/`, then ran `cargo clean` | Preserve the usable alpha while returning generated compiler space to the nearly full host. | `.app`, DMG, and CLI revalidated; 1.9 GiB build cache removed; artifacts use 53 MiB. |
