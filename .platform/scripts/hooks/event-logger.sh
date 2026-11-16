#!/usr/bin/env bash
# event-logger.sh — append lean AI agent events to .platform/events.jsonl
#
# Invoked by provider hooks/wrappers via stdin.
# Fail-open: errors never block a tool call.
#
# UserPromptSubmit events are dropped except /skill invocations.
# Raw hook payloads are never stored.

set -u
[[ -d ".platform" ]] || exit 0

log_file=".platform/events.jsonl"
mkdir -p "$(dirname "$log_file")" 2>/dev/null || exit 0

input="$(cat 2>/dev/null)"
[[ -n "$input" ]] || exit 0

ts="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)" || exit 0
provider="${AGENTBOARD_PROVIDER:-claude}"

_json_string_field() {
  local field="$1"
  printf '%s' "$input" | awk -v field="$field" '
    match($0, "\"" field "\"[[:space:]]*:[[:space:]]*\"[^\"]*\"") {
      s = substr($0, RSTART, RLENGTH)
      sub(".*\"" field "\"[[:space:]]*:[[:space:]]*\"", "", s)
      sub(/".*/, "", s)
      print s
      exit
    }
  '
}

_json_first_string_field() {
  local value field
  for field in "$@"; do
    value="$(_json_string_field "$field")"
    if [[ -n "$value" ]]; then
      printf '%s\n' "$value"
      return 0
    fi
  done
  return 1
}

_safe_session_id() {
  local value="$1"
  value="${value##*/}"
  value="${value##*\\}"
  value="${value%.jsonl}"
  value="${value%.json}"
  value="$(printf '%s' "$value" | tr -c '[:alnum:]_.:-' '_')"
  value="${value##_}"
  value="${value%%_}"
  printf '%s' "${value:0:160}"
}

_derive_session_id() {
  local raw
  raw="$(_json_first_string_field \
    "session_id" "sessionId" \
    "conversation_id" "conversationId" \
    "thread_id" "threadId" \
    "transcript_id" "transcriptId" \
    "transcript_path" "transcriptPath" 2>/dev/null || true)"
  if [[ -z "$raw" ]]; then
    raw="${AGENTBOARD_SESSION_ID:-}"
  fi
  raw="$(_safe_session_id "$raw")"
  if [[ -n "$raw" ]]; then
    printf '%s\n' "$raw"
  else
    printf '%s-ppid-%s\n' "$provider" "$PPID"
  fi
}

_jsesc() {
  printf '%s' "$1" | awk '{ gsub(/\\/, "\\\\"); gsub(/"/, "\\\""); printf "%s", $0 }'
}

_refresh_session_snapshot() {
  local sid="$1"
  [[ -n "$sid" ]] || return 0
  [[ -f ".platform/scripts/hooks/session-snapshot.js" ]] || return 0
  command -v node >/dev/null 2>&1 || return 0
  local runtime_hook_dir runtime_snapshot
  runtime_hook_dir=".platform/runtime/agentboard/node-hooks"
  runtime_snapshot="${runtime_hook_dir}/session-snapshot.cjs"
  mkdir -p "$runtime_hook_dir" 2>/dev/null || return 0
  cp ".platform/scripts/hooks/session-snapshot.js" "$runtime_snapshot" 2>/dev/null || return 0
  local payload
  payload="{\"session_id\":\"$(_jsesc "$sid")\",\"provider\":\"$(_jsesc "$provider")\",\"cwd\":\"$(_jsesc "$(pwd)")\"}"
  printf '%s' "$payload" \
    | AGENTBOARD_PROVIDER="$provider" \
      AGENTBOARD_SESSION_ID="$sid" \
      AGENTBOARD_MODEL= \
      AGENTBOARD_CWD="$(pwd)" \
      node "$runtime_snapshot" >/dev/null 2>&1 || true
}

hook_event="$(_json_string_field "hook_event_name")"
if [[ "$hook_event" == "UserPromptSubmit" ]]; then
  _prompt="$(_json_string_field "prompt")"
  if [[ "$_prompt" == /* ]]; then
    _skill="${_prompt%%[[:space:]]*}"
    _skill="${_skill#/}"
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null)" || exit 0
    _session_id="$(_derive_session_id)"
    _refresh_session_snapshot "$_session_id"
    printf '{"ts":"%s","provider":"%s","stream":"","tool":"Skill","skill":"%s","session_id":"%s"}\n' \
      "$ts" "$(_jsesc "$provider")" "$(_jsesc "$_skill")" "$(_jsesc "$_session_id")" >> "$log_file" 2>/dev/null
  fi
  exit 0
fi

_brief_primary_stream() {
  local brief=".platform/work/BRIEF.md" slug
  [[ -f "$brief" ]] || return 1
  slug="$(sed -n 's/^\*\*Stream file:\*\* `work\/\([^`]*\)\.md`$/\1/p' "$brief")"
  slug="${slug%%$'\n'*}"
  [[ -n "$slug" && -f ".platform/work/${slug}.md" ]] || return 1
  printf '%s\n' "$slug"
}

_active_streams() {
  local active=".platform/work/ACTIVE.md"
  [[ -f "$active" ]] || return 0
  awk -F'|' '
    function trim(s) { gsub(/^[ \t]+|[ \t]+$/, "", s); return s }
    {
      slug = trim($2)
      status = trim($4)
      if (slug == "" || slug == "Stream" || slug == "_(none)_" || slug ~ /^-+$/) next
      if (status == "" || status == "done" || status == "archived" || status == "closed") next
      print slug
    }
  ' "$active"
}

_stream_map_file=".platform/.session-streams.tsv"
_session_stream_lookup() {
  local session_id="$1"
  [[ -n "$session_id" && -f "$_stream_map_file" ]] || return 1
  awk -F'\t' -v session_id="$session_id" '
    $1 == session_id { print $2; found = 1; exit }
    END { exit found ? 0 : 1 }
  ' "$_stream_map_file"
}

_remember_session_stream() {
  local session_id="$1" stream_slug="$2" tmp
  [[ -n "$session_id" && -n "$stream_slug" ]] || return 0
  # First-write-wins: don't overwrite existing mapping (prevents cross-session contamination)
  if _session_stream_lookup "$session_id" 2>/dev/null; then return 0; fi
  tmp="$(mktemp 2>/dev/null)" || return 0
  [[ -f "$_stream_map_file" ]] && cat "$_stream_map_file" > "$tmp"
  printf '%s\t%s\n' "$session_id" "$stream_slug" >> "$tmp"
  mv "$tmp" "$_stream_map_file" 2>/dev/null || rm -f "$tmp"
}

_resolve_stream() {
  local explicit_stream="${1:-}" session_id="${2:-}" brief_stream active_streams active_count
  if [[ -n "$explicit_stream" && -f ".platform/work/${explicit_stream}.md" ]]; then
    printf '%s\n' "$explicit_stream"; return 0
  fi
  if [[ -n "$session_id" ]]; then
    local remembered
    remembered="$(_session_stream_lookup "$session_id" 2>/dev/null || true)"
    if [[ -n "$remembered" && -f ".platform/work/${remembered}.md" ]]; then
      printf '%s\n' "$remembered"; return 0
    fi
  fi
  brief_stream="$(_brief_primary_stream 2>/dev/null || true)"
  if [[ -n "$brief_stream" ]]; then printf '%s\n' "$brief_stream"; return 0; fi
  active_streams="$(_active_streams)"
  active_count="$(printf '%s\n' "$active_streams" | awk 'NF { count++ } END { print count + 0 }')"
  if [[ "$active_count" -eq 1 ]]; then printf '%s\n' "$active_streams"; return 0; fi
  return 1
}

session_id="$(_derive_session_id)"
payload_stream="$(_json_string_field "stream")"
stream="$(_resolve_stream "${AGENTBOARD_STREAM:-$payload_stream}" "$session_id" 2>/dev/null || true)"
if [[ -n "$stream" && -n "$session_id" ]]; then
  _remember_session_stream "$session_id" "$stream"
fi
_refresh_session_snapshot "$session_id"

tool="$(_json_string_field "tool_name")"

provider_e="$(_jsesc "$provider")"
stream_e="$(_jsesc "$stream")"
tool_e="$(_jsesc "$tool")"
hook_e="$(_jsesc "$hook_event")"

agent_id="${AGENTBOARD_AGENT_ID:-$(_json_first_string_field "agent_id" "agentId" "subagent_id" "subagentId" "agent_path" "agentPath" 2>/dev/null || true)}"
agent_label="${AGENTBOARD_AGENT_LABEL:-$(_json_first_string_field "agent_label" "agentLabel" "label" "subagent_type" "subagentType" "agent_type" "agentType" 2>/dev/null || true)}"
parent_session_id="${AGENTBOARD_PARENT_SESSION_ID:-$(_json_first_string_field "parent_session_id" "parentSessionId" 2>/dev/null || true)}"

_agent_attrs() {
  local attrs=""
  [[ -n "${agent_id:-}" ]] && attrs="${attrs},\"agent_id\":\"$(_jsesc "$agent_id")\""
  [[ -n "${agent_label:-}" ]] && attrs="${attrs},\"agent_label\":\"$(_jsesc "$agent_label")\""
  [[ -n "${parent_session_id:-}" ]] && attrs="${attrs},\"parent_session_id\":\"$(_jsesc "$parent_session_id")\""
  printf '%s' "$attrs"
}

_append_event_payload() {
  local payload="$1"
  if command -v flock >/dev/null 2>&1; then
    (
      flock -w 1 9 || exit 0
      printf '%s\n' "$payload" >&9
    ) 9>>"$log_file" 2>/dev/null
  else
    printf '%s\n' "$payload" >> "$log_file" 2>/dev/null
  fi
}

_bash_is_observational() {
  local cmd="$1"
  case "$cmd" in
    git\ status*|git\ diff*|git\ show*|git\ log*|git\ branch*|git\ rev-parse*|git\ commit*|\
    rg\ *|grep\ *|find\ *|ls\ *|pwd|cat\ *|head\ *|tail\ *|sed\ *|awk\ *|\
    echo\ *|printf\ *|test\ *|\[\ *) return 0 ;;
  esac
  return 1
}

_emit_bash_file_changes() {
  local cmd="$1"
  [[ -n "$cmd" ]] || return 0
  _bash_is_observational "$cmd" && return 0
  command -v git >/dev/null 2>&1 || return 0
  git rev-parse --is-inside-work-tree >/dev/null 2>&1 || return 0
  local dirty_files file file_e sid_e attrs
  dirty_files="$(
    {
      git diff --name-only --diff-filter=ACDMRTUXB HEAD -- 2>/dev/null
      git ls-files --others --exclude-standard 2>/dev/null
    } | awk 'NF && !seen[$0]++ && $0 !~ /^\.platform\// { print }' | head -80
  )"
  [[ -n "$dirty_files" ]] || return 0
  sid_e="$(_jsesc "${session_id:-}")"
  attrs="$(_agent_attrs)"
  while IFS= read -r file; do
    [[ -n "$file" ]] || continue
    file_e="$(_jsesc "$file")"
    _append_event_payload "{\"ts\":\"$ts\",\"provider\":\"$provider_e\",\"stream\":\"$stream_e\",\"hook_event_name\":\"FileChange\",\"tool\":\"Edit\",\"session_id\":\"$sid_e\",\"file_path\":\"$file_e\",\"file\":\"$file_e\"$attrs}"
  done <<< "$dirty_files"
}

if [[ "${AGENTBOARD_HOOK_TYPE:-}" == "agent_start" ]]; then
  _label="$(_json_string_field "label")"
  _subtype="$(_json_string_field "subagent_type")"
  _role=""
  _skill=""
  if [[ "$_label" == *"role:"* ]]; then
    _role="$(printf '%s' "$_label" | sed 's/.*role:\([^·|]*\).*/\1/' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  fi
  if [[ "$_label" == *"skill:"* ]]; then
    _skill="$(printf '%s' "$_label" | sed 's/.*skill:\([^·|]*\).*/\1/' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  fi
  _task="${_label:-${_subtype:-sub-agent}}"
  agent_id="${agent_id:-$_task}"
  agent_label="${agent_label:-$_task}"
  printf '{"ts":"%s","provider":"%s","stream":"%s","tool":"AgentStart","label":"%s","role":"%s","skill":"%s","session_id":"%s"%s}\n' \
    "$ts" "$provider_e" "$stream_e" \
    "$(_jsesc "$_task")" "$(_jsesc "$_role")" "$(_jsesc "$_skill")" \
    "$(_jsesc "$session_id")" "$(_agent_attrs)" >> "$log_file" 2>/dev/null
  exit 0
fi

if [[ "${AGENTBOARD_HOOK_TYPE:-}" == "agent_done" ]]; then
  _label="$(_json_string_field "label")"
  _task="${_label:-sub-agent}"
  agent_id="${agent_id:-$_task}"
  agent_label="${agent_label:-$_task}"
  printf '{"ts":"%s","provider":"%s","stream":"%s","tool":"AgentDone","label":"%s","session_id":"%s"%s}\n' \
    "$ts" "$provider_e" "$stream_e" \
    "$(_jsesc "$_task")" "$(_jsesc "$session_id")" "$(_agent_attrs)" >> "$log_file" 2>/dev/null
  exit 0
fi

if [[ "$tool" == "Bash" ]]; then
  _cmd_peek="$(_json_string_field "command")"
  case "$_cmd_peek" in
    ab\ *|agentboard\ *) exit 0 ;;
  esac
fi

case "$hook_event" in
  SessionStart|SessionEnd|FileChange|Reason)
    _sid_e="$(_jsesc "${session_id:-}")"
    _agent_attrs="$(_agent_attrs)"
    _fp="$(_json_string_field "file_path")"
    if [[ -n "$_fp" ]]; then
      _fp_e="$(_jsesc "$_fp")"
      if [[ "$hook_event" == "FileChange" ]]; then
        _payload="{\"ts\":\"$ts\",\"provider\":\"$provider_e\",\"stream\":\"$stream_e\",\"hook_event_name\":\"$hook_e\",\"tool\":\"Edit\",\"session_id\":\"$_sid_e\",\"file_path\":\"$_fp_e\",\"file\":\"$_fp_e\"$_agent_attrs}"
      else
        _payload="{\"ts\":\"$ts\",\"provider\":\"$provider_e\",\"stream\":\"$stream_e\",\"hook_event_name\":\"$hook_e\",\"session_id\":\"$_sid_e\",\"file_path\":\"$_fp_e\"$_agent_attrs}"
      fi
    else
      _payload="{\"ts\":\"$ts\",\"provider\":\"$provider_e\",\"stream\":\"$stream_e\",\"hook_event_name\":\"$hook_e\",\"session_id\":\"$_sid_e\"$_agent_attrs}"
    fi
    ;;
  *)
    detail_key=""
    detail_val=""
    case "$tool" in
      Read)
        _fp="$(_json_string_field "file_path")"
        case "$_fp" in
          */.claude/skills/*/SKILL.md|*/.claude/skills/*/*|*/.agents/skills/*/SKILL.md|*/.agents/skills/*/*)
            _skill_name="${_fp#*/.claude/skills/}"
            [[ "$_skill_name" == "$_fp" ]] && _skill_name="${_fp#*/.agents/skills/}"
            _skill_name="${_skill_name%%/*}"
            if [[ -n "$_skill_name" ]]; then
              printf '{"ts":"%s","provider":"%s","stream":"%s","tool":"Skill","skill":"%s","session_id":"%s"%s}\n' \
                "$ts" "$provider_e" "$stream_e" "$(_jsesc "$_skill_name")" "$(_jsesc "$session_id")" "$(_agent_attrs)" >> "$log_file" 2>/dev/null
            fi
            exit 0
            ;;
          */.platform/roles/*.md)
            _role_slug="$(printf '%s' "$_fp" | sed 's|.*\.platform/roles/\([^/]*\)\.md|\1|')"
            if [[ -n "$_role_slug" ]]; then
              printf '{"ts":"%s","provider":"%s","stream":"%s","tool":"RoleAdopt","role":"%s","session_id":"%s"%s}\n' \
                "$ts" "$provider_e" "$stream_e" "$(_jsesc "$_role_slug")" "$(_jsesc "$session_id")" "$(_agent_attrs)" >> "$log_file" 2>/dev/null
            fi
            exit 0
            ;;
          *)
            exit 0  # other reads — not useful activity
            ;;
        esac
        ;;
      Edit|Write|MultiEdit|NotebookEdit)
        _fp="$(_json_string_field "file_path")"
        _rel="${_fp##"$(pwd)/"}"
        case "$_rel" in .platform/*) exit 0 ;; esac
        if [[ -n "$_fp" ]]; then
          detail_key="file"
          detail_val="$_rel"
        fi
        ;;
      Bash)
        _cmd="$(_json_string_field "command")"
        case "$_cmd" in
          echo\ *|printf\ *|cat\ *|ls\ *|cd\ *|pwd|true|false|:|\
          mkdir\ *|rm\ *|mv\ *|cp\ *|touch\ *|chmod\ *|wc\ *|head\ *|tail\ *|\
          sed\ *|awk\ *|grep\ *|find\ *|sort\ *|uniq\ *|test\ *|\[\ *|\
          export\ *|source\ *|\.\ *|read\ *) exit 0 ;;
          *) ;;
        esac
        if [[ -n "$_cmd" ]]; then
          detail_key="cmd"
          detail_val="${_cmd:0:120}"
          _emit_bash_file_changes "$_cmd"
        fi
        ;;
      Skill)
        _sk_type="$(_json_string_field "subagent_type")"
        _sk_name="${_sk_type:-$(_json_string_field "type")}"
        if [[ -n "$_sk_name" ]]; then
          printf '{"ts":"%s","provider":"%s","stream":"%s","tool":"Skill","skill":"%s","session_id":"%s"%s}\n' \
            "$ts" "$provider_e" "$stream_e" "$(_jsesc "$_sk_name")" "$(_jsesc "${session_id:-}")" "$(_agent_attrs)" >> "$log_file" 2>/dev/null
        fi
        exit 0
        ;;
      Agent)
        _label="$(_json_string_field "label")"
        _subtype="$(_json_string_field "subagent_type")"
        _agent_id="${_label:-${_subtype:-sub-agent}}"
        detail_key="agent"
        detail_val="$_agent_id"
        ;;
      WebSearch|WebFetch)
        exit 0
        ;;
    esac
    _sid_e="$(_jsesc "${session_id:-}")"
    _agent_attrs="$(_agent_attrs)"
    if [[ -n "$detail_key" && -n "$detail_val" ]]; then
      detail_e="$(_jsesc "$detail_val")"
      _payload="{\"ts\":\"$ts\",\"provider\":\"$provider_e\",\"stream\":\"$stream_e\",\"tool\":\"$tool_e\",\"$detail_key\":\"$detail_e\",\"session_id\":\"$_sid_e\"$_agent_attrs}"
    else
      _payload="{\"ts\":\"$ts\",\"provider\":\"$provider_e\",\"stream\":\"$stream_e\",\"tool\":\"$tool_e\",\"session_id\":\"$_sid_e\"$_agent_attrs}"
    fi
    ;;
esac

_port_file=".platform/.daemon-port"
_daemon_ok=0
if [[ -f "$_port_file" ]] && command -v curl >/dev/null 2>&1; then
  _port="$(cat "$_port_file" 2>/dev/null)"
  if [[ "$_port" =~ ^[0-9]+$ ]]; then
    if curl -sf -m 1 -X POST "http://127.0.0.1:$_port/event" \
        -H 'Content-Type: application/json' \
        -d "$_payload" >/dev/null 2>&1; then
      _daemon_ok=1
    fi
  fi
fi

if (( _daemon_ok == 0 )); then
  if command -v flock >/dev/null 2>&1; then
    (
      flock -w 1 9 || exit 0
      printf '%s\n' "$_payload" >&9
    ) 9>>"$log_file" 2>/dev/null
  else
    printf '%s\n' "$_payload" >> "$log_file" 2>/dev/null
  fi
fi

exit 0
