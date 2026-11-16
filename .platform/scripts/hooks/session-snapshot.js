#!/usr/bin/env node
/**
 * session-snapshot.js — write Agentboard live session JSON for provider wrappers/hooks.
 *
 * Claude Code writes this shape from status-bridge.js. Codex/Gemini wrappers and
 * Codex native hooks call this helper so the VS Code dashboard can stay
 * provider-agnostic while reading the current repo's private runtime store.
 */

'use strict';

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const MAX_STDIN = 512 * 1024;

function readStdin() {
  try {
    if (process.stdin.isTTY) return {};
    const chunks = [];
    let total = 0;
    const buf = fs.readFileSync(0);
    total += buf.length;
    if (total > MAX_STDIN) return {};
    chunks.push(buf);
    const raw = Buffer.concat(chunks).toString('utf8').trim();
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

function get(obj, pathExpr) {
  let cur = obj;
  for (const part of pathExpr.split('.')) {
    if (cur == null || typeof cur !== 'object') return undefined;
    cur = cur[part];
  }
  return cur;
}

function firstString(...values) {
  for (const value of values) {
    if (typeof value === 'string' && value.trim()) return value.trim();
    if (typeof value === 'number' && Number.isFinite(value)) return String(value);
  }
  return '';
}

function safeSessionId(value) {
  let id = firstString(value);
  if (!id) return '';
  id = path.basename(id).replace(/\.(jsonl|json)$/i, '');
  id = id.replace(/[^a-zA-Z0-9_.:-]+/g, '_').replace(/^_+|_+$/g, '');
  return id.slice(0, 160);
}

function findProjectRoot(cwd) {
  let dir = cwd || process.cwd();
  for (let i = 0; i < 30; i++) {
    if (fs.existsSync(path.join(dir, '.platform')) || fs.existsSync(path.join(dir, '.git'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return cwd || process.cwd();
}

function readJson(file) {
  try { return JSON.parse(fs.readFileSync(file, 'utf8')); } catch { return {}; }
}

function writeJsonAtomic(file, data) {
  const tmp = `${file}.tmp.${process.pid}`;
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(tmp, JSON.stringify(data, null, 2));
  fs.renameSync(tmp, file);
}

function runtimeDirForRoot(root) {
  return path.join(root, '.platform', 'runtime', 'agentboard');
}

function sessionsDirForRoot(root) {
  return path.join(runtimeDirForRoot(root), 'sessions');
}

function gitBranch(root) {
  try {
    return execSync('git rev-parse --abbrev-ref HEAD', { cwd: root, timeout: 500, encoding: 'utf8' }).trim();
  } catch {
    return '';
  }
}

function deriveSessionId(payload, provider) {
  return safeSessionId(firstString(
    payload.session_id,
    payload.sessionId,
    payload.conversation_id,
    payload.conversationId,
    payload.thread_id,
    payload.threadId,
    payload.transcript_id,
    payload.transcriptId,
    payload.transcript_path,
    payload.transcriptPath,
    get(payload, 'session.id'),
    get(payload, 'conversation.id'),
    get(payload, 'thread.id'),
    process.env.AGENTBOARD_SESSION_ID,
  )) || `${provider || 'agent'}-${process.ppid || process.pid}`;
}

function deriveModel(payload) {
  const model = payload.model;
  return firstString(
    typeof model === 'string' ? model : '',
    get(model || {}, 'api_name'),
    get(model || {}, 'display_name'),
    payload.model_name,
    payload.modelName,
    payload.model_id,
    payload.modelId,
    process.env.AGENTBOARD_MODEL,
  );
}

function numberOrNull(...values) {
  for (const value of values) {
    if (typeof value === 'number' && Number.isFinite(value)) return value;
    if (typeof value === 'string' && value.trim() && Number.isFinite(Number(value))) return Number(value);
  }
  return null;
}

function writeSnapshot(payload = {}, opts = {}) {
  const provider = firstString(opts.provider, process.env.AGENTBOARD_PROVIDER, payload.provider) || 'unknown';
  const cwd = firstString(opts.cwd, process.env.AGENTBOARD_CWD, get(payload, 'workspace.current_dir'), payload.cwd) || process.cwd();
  const root = findProjectRoot(cwd);
  const sessionId = firstString(opts.sessionId, deriveSessionId(payload, provider));
  const now = new Date().toISOString();
  const sessionsDir = sessionsDirForRoot(root);
  const sessionPath = path.join(sessionsDir, `${sessionId}.json`);
  const hudPath = path.join(root, 'agentboard.hud-status.json');
  const existing = readJson(sessionPath);
  const existingHud = readJson(hudPath);
  const existingCtx = existing.context || {};
  const existingCost = existing.cost || {};
  const existingHudCtx = existingHud.context || {};
  const existingHudStartedAt = existingHudCtx.session_id === sessionId ? existingHudCtx.started_at : '';
  const model = firstString(opts.model, deriveModel(payload), existingCtx.model, provider);
  const startedAt = firstString(
    opts.startedAt,
    payload.started_at,
    payload.startedAt,
    existingCtx.started_at,
    existingHudStartedAt,
  ) || now;

  const ctxRemaining = numberOrNull(
    opts.contextRemainingPct,
    get(payload, 'context_window.remaining_percentage'),
    payload.context_remaining_pct,
    payload.contextRemainingPct,
    existingCtx.context_remaining_pct,
  );
  const ctxTokens = numberOrNull(
    opts.contextTokens,
    get(payload, 'context_window.current_tokens'),
    payload.context_tokens,
    payload.contextTokens,
    existingCtx.context_tokens,
  );
  const costUsd = numberOrNull(
    opts.costUsd,
    get(payload, 'cost.total_cost_usd'),
    payload.total_cost_usd,
    payload.cost_usd,
    existingCost.session_usd,
  );
  const shellPid = numberOrNull(
    opts.shellPid,
    process.env.AGENTBOARD_SHELL_PID,
    payload.shell_pid,
    payload.shellPid,
    existing._shell_pid,
  ) || 0;

  const snapshot = {
    ...existing,
    provider,
    last_updated: now,
    context: {
      ...existingCtx,
      provider,
      model,
      session_id: sessionId,
      branch: firstString(existingCtx.branch, gitBranch(root)),
      started_at: startedAt,
      context_remaining_pct: ctxRemaining,
      context_tokens: ctxTokens,
    },
    cost: {
      ...existingCost,
      session_usd: costUsd,
      session_tokens: ctxTokens,
    },
    active_agents: [{
      role: get(existing, 'active_agents.0.role') || provider,
      model,
      started_at: startedAt,
      session_id: sessionId,
      provider,
    }],
    _root: root,
    _session_id: sessionId,
    _last_updated: now,
    _shell_pid: shellPid,
  };

  writeJsonAtomic(sessionPath, snapshot);
  writeJsonAtomic(hudPath, snapshot);

  try {
    const cutoff = Date.now() - 8 * 60 * 60 * 1000;
    for (const file of fs.readdirSync(sessionsDir)) {
      if (!file.endsWith('.json')) continue;
      const fp = path.join(sessionsDir, file);
      try { if (fs.statSync(fp).mtimeMs < cutoff) fs.unlinkSync(fp); } catch {}
    }
  } catch {}

  return snapshot;
}

if (require.main === module) {
  try {
    const payload = readStdin();
    const snapshot = writeSnapshot(payload);
    if (process.argv.includes('--print')) process.stdout.write(JSON.stringify(snapshot));
  } catch {
    process.exit(0);
  }
}

module.exports = { writeSnapshot, deriveSessionId, findProjectRoot, runtimeDirForRoot, sessionsDirForRoot };
