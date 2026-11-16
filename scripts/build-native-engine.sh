#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
destination="${1:?destination app bundle path is required}"
configuration="${CONFIGURATION:-Debug}"
architecture="${CURRENT_ARCH:-}"
if [[ -z "$architecture" || "$architecture" == "undefined_arch" ]]; then
  architecture="${NATIVE_ARCH_ACTUAL:-$(uname -m)}"
fi

case "$architecture" in
  arm64) rust_target="aarch64-apple-darwin" ;;
  x86_64) rust_target="x86_64-apple-darwin" ;;
  *) echo "Unsupported macOS architecture: $architecture" >&2; exit 2 ;;
esac

profile_dir="debug"
if [[ "$configuration" == "Release" ]]; then
  profile_dir="release"
fi

if [[ -z "${MAC_CLEANER_ENGINE_PATH:-}" || -z "${MAC_CLEANER_MCP_PATH:-}" ]]; then
  cargo_args=(build --target "$rust_target")
  if [[ "$configuration" == "Release" ]]; then
    cargo_args+=(--release)
  fi
fi

if [[ -n "${MAC_CLEANER_ENGINE_PATH:-}" ]]; then
  engine_path="$MAC_CLEANER_ENGINE_PATH"
else
  (cd "$repo_root" && cargo "${cargo_args[@]}" -p mac-cleaner-engine)
  engine_path="$repo_root/target/$rust_target/$profile_dir/mac-cleaner-engine"
fi

if [[ -n "${MAC_CLEANER_MCP_PATH:-}" ]]; then
  mcp_path="$MAC_CLEANER_MCP_PATH"
else
  (cd "$repo_root" && cargo "${cargo_args[@]}" -p mac-cleaner-mcp)
  mcp_path="$repo_root/target/$rust_target/$profile_dir/mac-cleaner-mcp"
fi

helpers_dir="$destination/Contents/Helpers"
mkdir -p "$helpers_dir"
/usr/bin/ditto "$engine_path" "$helpers_dir/mac-cleaner-engine"
/bin/chmod 755 "$helpers_dir/mac-cleaner-engine"
/usr/bin/ditto "$mcp_path" "$helpers_dir/mac-cleaner-mcp"
/bin/chmod 755 "$helpers_dir/mac-cleaner-mcp"
