#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
native_dir="$repo_root/apps/mac-cleaner-macos"
derived_dir="$repo_root/target/native-dev"

xcodebuild \
  -project "$native_dir/MacCleanerNative.xcodeproj" \
  -scheme MacCleanerNative \
  -configuration Debug \
  -derivedDataPath "$derived_dir" \
  CODE_SIGNING_ALLOWED=NO ONLY_ACTIVE_ARCH=YES \
  build
open "$derived_dir/Build/Products/Debug/Mac Cleaner.app"
