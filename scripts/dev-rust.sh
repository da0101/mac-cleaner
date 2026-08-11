#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -x /opt/homebrew/opt/node@22/bin/node ]]; then
  export PATH="/opt/homebrew/opt/node@22/bin:$PATH"
fi
app_dir="$repo_root/apps/mac-cleaner-desktop"
tauri_cli="$app_dir/ui/node_modules/.bin/tauri"

cd "$app_dir"
exec "$tauri_cli" dev
