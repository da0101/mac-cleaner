#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
native_dir="$repo_root/apps/mac-cleaner-macos"
derived_dir="${TMPDIR:-/tmp}/mac-cleaner-native-tests"

cd "$repo_root"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
xcodebuild test \
  -project "$native_dir/MacCleanerNative.xcodeproj" \
  -scheme MacCleanerNative \
  -configuration Debug \
  -derivedDataPath "$derived_dir" \
  -destination 'platform=macOS,arch=arm64' \
  CODE_SIGNING_ALLOWED=NO ONLY_ACTIVE_ARCH=YES
