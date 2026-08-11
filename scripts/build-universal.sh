#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -x /opt/homebrew/opt/node@22/bin/node ]]; then
  export PATH="/opt/homebrew/opt/node@22/bin:$PATH"
fi
app_dir="$repo_root/apps/mac-cleaner-desktop"
tauri_cli="$app_dir/ui/node_modules/.bin/tauri"
output_dir="$repo_root/target/universal-cli"

cd "$repo_root"
cargo build --release -p mac-cleaner --target aarch64-apple-darwin
cargo build --release -p mac-cleaner --target x86_64-apple-darwin
mkdir -p "$output_dir"
lipo -create \
  "$repo_root/target/aarch64-apple-darwin/release/mac-cleaner" \
  "$repo_root/target/x86_64-apple-darwin/release/mac-cleaner" \
  -output "$output_dir/mac-cleaner"

cd "$app_dir"
"$tauri_cli" build --target universal-apple-darwin
lipo -info "$output_dir/mac-cleaner"
