#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
native_dir="$repo_root/apps/mac-cleaner-macos"
universal_dir="$repo_root/target/universal-macos"
derived_dir="$repo_root/target/native-derived"
release_dir="$repo_root/target/native-release"

cd "$repo_root"
for target in aarch64-apple-darwin x86_64-apple-darwin; do
  cargo build --release -p mac-cleaner -p mac-cleaner-engine -p mac-cleaner-mcp --target "$target"
done

mkdir -p "$universal_dir" "$release_dir"
for binary in mac-cleaner mac-cleaner-engine mac-cleaner-mcp; do
  lipo -create \
    "$repo_root/target/aarch64-apple-darwin/release/$binary" \
    "$repo_root/target/x86_64-apple-darwin/release/$binary" \
    -output "$universal_dir/$binary"
done

rm -rf "$derived_dir"
MAC_CLEANER_ENGINE_PATH="$universal_dir/mac-cleaner-engine" \
MAC_CLEANER_MCP_PATH="$universal_dir/mac-cleaner-mcp" \
xcodebuild \
  -project "$native_dir/MacCleanerNative.xcodeproj" \
  -scheme MacCleanerNative \
  -configuration Release \
  -derivedDataPath "$derived_dir" \
  -destination 'generic/platform=macOS' \
  ARCHS='arm64 x86_64' ONLY_ACTIVE_ARCH=NO CODE_SIGNING_ALLOWED=NO \
  build

app_source="$derived_dir/Build/Products/Release/Mac Cleaner.app"
app_output="$release_dir/Mac Cleaner.app"
rm -rf "$app_output"
/usr/bin/ditto "$app_source" "$app_output"
rm -f "$release_dir/Mac-Cleaner-unsigned.dmg"
hdiutil create -volname "Mac Cleaner" -srcfolder "$app_output" -ov -format UDZO \
  "$release_dir/Mac-Cleaner-unsigned.dmg"

embedded_engine="$app_output/Contents/Helpers/mac-cleaner-engine"
embedded_mcp="$app_output/Contents/Helpers/mac-cleaner-mcp"
test -x "$embedded_engine"
test -x "$embedded_mcp"
engine_status="$("$embedded_engine" status)"
case "$engine_status" in
  *'"protocol_version":1'*'"ok":true'*) ;;
  *)
    echo "Embedded engine smoke test failed: $engine_status" >&2
    exit 1
    ;;
esac

mcp_initialize='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"mac-cleaner-build","version":"1.0"}}}'
mcp_initialized='{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
mcp_list='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
mcp_smoke="$(printf '%s\n%s\n%s\n' "$mcp_initialize" "$mcp_initialized" "$mcp_list" | "$embedded_mcp")"
case "$mcp_smoke" in
  *'"name":"analyze_storage"'*'"name":"latest_storage_report"'*'"readOnlyHint":true'*) ;;
  *)
    echo "Embedded MCP smoke test failed: $mcp_smoke" >&2
    exit 1
    ;;
esac

lipo -info "$universal_dir/mac-cleaner"
lipo -info "$app_output/Contents/MacOS/Mac Cleaner"
lipo -info "$embedded_engine"
lipo -info "$embedded_mcp"
