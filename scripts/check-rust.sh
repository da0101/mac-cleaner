#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -x /opt/homebrew/opt/node@22/bin/node ]]; then
  export PATH="/opt/homebrew/opt/node@22/bin:$PATH"
fi
cd "$repo_root"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm audit --prefix apps/mac-cleaner-desktop/ui --audit-level=moderate
npm test --prefix apps/mac-cleaner-desktop/ui
npm run build --prefix apps/mac-cleaner-desktop/ui
