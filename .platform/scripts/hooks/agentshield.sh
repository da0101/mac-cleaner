#!/usr/bin/env bash
# agentshield.sh — PreToolUse security gate for Claude Code (agentboard kit).
#
# Intercepts dangerous Bash commands and writes to .env files before they run.
# Fail-open: any parse failure exits 0 silently — this is a guardrail, not a
# firewall. The worst case is one missed prompt, not a blocked workflow.
#
# To install, add to .claude/settings.json under hooks.PreToolUse:
#   { "matcher": "", "hooks": [{ "type": "command",
#     "command": "bash .platform/scripts/hooks/agentshield.sh" }] }
#
# Input:  JSON on stdin — {"tool_name":"...","tool_input":{...},...}
# Output: {"permissionDecision":"ask","permissionDecisionReason":"..."} if
#         dangerous; exit 0 silently otherwise.

set -uo pipefail

INPUT="$(cat 2>/dev/null || true)"
[[ -n "$INPUT" ]] || exit 0

# Extract tool_name — no jq dependency; grep is sufficient for a known JSON key.
TOOL="$(printf '%s' "$INPUT" | grep -o '"tool_name"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"[^"]*"$' | tr -d '"')"

_ask() {
  printf '{"permissionDecision":"ask","permissionDecisionReason":"%s"}\n' "$1"
  exit 0
}

# ── Bash tool checks ────────────────────────────────────────────────────────
if [ "$TOOL" = "Bash" ]; then
  CMD="$(printf '%s' "$INPUT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"[^"]*"$' | tr -d '"')"
  [[ -n "$CMD" ]] || exit 0

  # (a) curl/wget piped to bash/sh, or eval with command substitution
  if printf '%s' "$CMD" | grep -qE '(curl|wget)[^|]*\|[[:space:]]*(bash|sh)|eval[^(]*\$\('; then
    _ask "agentshield: remote code execution pattern detected (curl/wget|bash or eval+\$()). Verify this is intentional."
  fi

  # (b) reading secret files at absolute paths outside the current workspace
  if printf '%s' "$CMD" | grep -qE '(cat|grep)[[:space:]].*/(\.env|id_rsa|[^[:space:]]+\.(pem|key))'; then
    # Allow paths that start with PWD (workspace-local files are fine)
    WORKSPACE="${PWD:-}"
    if [ -n "$WORKSPACE" ] && printf '%s' "$CMD" | grep -qE '(cat|grep)[[:space:]].*'"$WORKSPACE"; then
      : # workspace-local — allow
    else
      _ask "agentshield: reading a secret file (\.env / id_rsa / \.pem / \.key) outside the workspace. Confirm this is intentional."
    fi
  fi

  # (c) wide recursive deletes of / or ~
  if printf '%s' "$CMD" | grep -qE 'rm[[:space:]]+-[rRfF]*[[:space:]]+(\/|~)[[:space:]]*$'; then
    _ask "agentshield: rm -rf / or rm -rf ~ detected — this would wipe the filesystem. Denying."
  fi

  # (d) base64 -d piped to bash/sh
  if printf '%s' "$CMD" | grep -qE 'base64[[:space:]]+-d[^|]*\|[[:space:]]*(bash|sh)'; then
    _ask "agentshield: base64-decoded payload piped to shell detected. Verify this is intentional."
  fi
fi

# ── Write / Edit tool checks ────────────────────────────────────────────────
if [ "$TOOL" = "Write" ] || [ "$TOOL" = "Edit" ]; then
  # Extract file_path from tool_input
  FPATH="$(printf '%s' "$INPUT" | grep -o '"file_path"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | grep -o '"[^"]*"$' | tr -d '"')"
  [[ -n "$FPATH" ]] || exit 0

  # Block writes to any .env file (but not .env.example)
  BASENAME="$(printf '%s' "$FPATH" | grep -o '[^/]*$')"
  if printf '%s' "$BASENAME" | grep -qE '^\.env$|^\.env\.[^e]|^\.env\.[^x]'; then
    if ! printf '%s' "$BASENAME" | grep -q '\.env\.example'; then
      _ask "agentshield: writing to $FPATH is blocked — secrets must not be committed. Use .env.example for templates and populate .env manually."
    fi
  fi
fi

exit 0
