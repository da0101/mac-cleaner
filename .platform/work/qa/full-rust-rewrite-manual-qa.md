# Full Rust rewrite manual QA

- Date: 2026-08-10
- Build: `feature/full-rust-rewrite`, uncommitted developer build
- Host: macOS, Apple Silicon, dark appearance
- Data boundary: real home was inspected only by read-only monitoring; no cleanup, restore, purge, or process action was executed

## Results

| Check | Result | Evidence / notes |
|---|---|---|
| Native launch | Pass | Tauri compiled and opened `target/debug/mac-cleaner-desktop` as a native accessory app. |
| Bundled UI render | Pass | Native window rendered the header, safety statement, memory/disk metrics, and process monitor without a localhost application server. |
| Live system snapshot | Pass | UI displayed plausible memory, disk, PID, RSS, and CPU values from the Rust monitor. |
| Developer-process protection | Pass | Code helper processes were labeled `Developer work protected`; the interface states it cannot signal, quit, or kill them. |
| Storage safety copy | Pass | Interface states automatic deletion is off and unknown/development paths stay protected. |
| Login startup | Issue found and fixed | QA observed a LaunchAgent pointing at `target/debug/mac-cleaner-desktop`. It was moved intact to `/tmp/Mac Cleaner.plist.qa-created-20260810`. The legacy Python LaunchAgent was not changed. Login-item controls are now disabled outside an installed `.app` and require native confirmation. |
| Login startup regression | Pass | Relaunched the patched developer build: the control displayed `Installed app only`, remained disabled/off, and no `Mac Cleaner.plist` was recreated. |
| Read-only CLI status | Pass | Returned memory/disk data, 12 process records, and 7 protected developer-process records. |
| Read-only CLI scan | Pass | Completed against the real home with zero candidates and one protected/unsafe skip; no cleanup command was run. |
| Native accessibility-driven interaction | Blocked by host | Both app-state and app-list inspection timed out with macOS Apple Event error `-1712`; no simulated click or key input succeeded. |
| Read-only screenshot inspection | Pass | A macOS screenshot was inspected locally; it was not added to the repository because it contains unrelated desktop windows. |
| Destructive UI flow | Not run by design | Exact cleanup, restore, and purge paths are covered by fake-home integration tests only. |
| Universal package | Pass | Built arm64+x86_64 CLI and `.app`; `lipo` reports both architectures. Built a universal DMG and verified its checksum with `hdiutil`. |
| Packaged app launch | Pass | Ad-hoc alpha `.app` launched directly, rendered live metrics, exposed (but did not enable) the installed-app login control, and created no LaunchAgent. |
| Product icon | Pass | Generated shield/sweep/sparkle mark was inspected at 128 px and from the packaged 1024 px ICNS; it is present in `Contents/Resources/icon.icns` and referenced by `CFBundleIconFile`. |
| Release signing | Blocked by credentials | Alpha bundle is ad-hoc signed (`TeamIdentifier` absent). Developer ID signing, notarization, and updater signing require release credentials and remain a ship gate. |

Preserved local outputs are under `artifacts/full-rust-rewrite-alpha/`. Cargo's generated 1.9 GiB release cache was removed after the `.app`, DMG, and CLI were copied and revalidated.

## Required release-device follow-up

- Grant the QA runner Screen Recording and Accessibility permission, then verify keyboard navigation, scan, plan cancellation, native cleanup cancellation, close-to-hide, tray reopen, single-instance focus, and installed-app login toggle.
- Repeat on Intel macOS and an Apple Silicon release machine using signed `.app` bundles.
- Execute cleanup/restore/purge only against a disposable fake home or APFS test volume.
