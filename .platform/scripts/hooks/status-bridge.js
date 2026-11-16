#!/usr/bin/env node
/**
 * status-bridge.js — agentboard statusLine hook
 *
 * Claude Code calls this on every assistant turn via the "statusLine" setting.
 * Stdin payload: { session_id, model, context_window, cost, workspace, ... }
 *
 * Writes agentboard.hud-status.json so the dashboard gets live data every turn.
 * Also outputs the terminal statusLine string that replaces Claude's built-in.
 */

'use strict';

const fs = require('fs');
const path = require('path');

const MAX_STDIN = 512 * 1024;
const TIMEOUT_MS = 3000;

function formatElapsed(isoTs) {
  if (!isoTs) return '';
  const s = Math.floor((Date.now() - new Date(isoTs).getTime()) / 1000);
  if (s < 60) return `${s}s`;
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m ${s % 60}s`;
  return `${Math.floor(m / 60)}h ${m % 60}m`;
}

function findProjectRoot(cwd) {
  let dir = cwd || process.cwd();
  for (let i = 0; i < 20; i++) {
    if (fs.existsSync(path.join(dir, '.platform')) || fs.existsSync(path.join(dir, '.git'))) {
      return dir;
    }
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return cwd || process.cwd();
}

function readHud(hudPath) {
  try { return JSON.parse(fs.readFileSync(hudPath, 'utf8')); } catch { return {}; }
}

function writeHudAtomic(hudPath, data) {
  const tmp = hudPath + '.tmp.' + process.pid;
  try {
    fs.writeFileSync(tmp, JSON.stringify(data, null, 2));
    fs.renameSync(tmp, hudPath);
  } catch { try { fs.unlinkSync(tmp); } catch {} }
}

function runtimeDirForRoot(root) {
  return path.join(root, '.platform', 'runtime', 'agentboard');
}

function sessionsDirForRoot(root) {
  return path.join(runtimeDirForRoot(root), 'sessions');
}

let input = '';
const timer = setTimeout(() => process.exit(0), TIMEOUT_MS);

process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => { if (input.length < MAX_STDIN) input += chunk; });
process.stdin.on('end', () => {
  clearTimeout(timer);
  try {
    const data = JSON.parse(input);
    const sessionId = data.session_id || '';
    const model = data.model?.display_name || data.model?.api_name || 'claude';
    const apiName = data.model?.api_name || '';
    const cwd = data.workspace?.current_dir || data.cwd || process.cwd();
    const root = findProjectRoot(cwd);
    const hudPath = path.join(root, 'agentboard.hud-status.json');

    // context window
    const ctxRemaining = data.context_window?.remaining_percentage ?? null;
    const ctxTokens = data.context_window?.current_tokens ?? null;

    // cost — Claude Code passes this directly to statusLine
    const costUsd = data.cost?.total_cost_usd ?? data.total_cost_usd ?? null;

    // session start time — read from per-session file first (survives multi-session HUD overwrites)
    const existing = readHud(hudPath);
    const sessionsDir = sessionsDirForRoot(root);
    const sessionFilePath = path.join(sessionsDir, sessionId + '.json');
    const existingSession = readHud(sessionFilePath);
    const sessionStartedAt = (existingSession.context?.session_id === sessionId && existingSession.context?.started_at)
      ? existingSession.context.started_at
      : (existing.context?.session_id === sessionId && existing.context?.started_at)
        ? existing.context.started_at
        : new Date().toISOString();

    const hud = {
      ...existing,
      last_updated: new Date().toISOString(),
      context: {
        ...(existing.context || {}),
        model: apiName || model,
        session_id: sessionId,
        branch: existing.context?.branch || (() => {
          try {
            return require('child_process').execSync('git rev-parse --abbrev-ref HEAD', { cwd: root, timeout: 500 }).toString().trim();
          } catch { return ''; }
        })(),
        started_at: sessionStartedAt,
        context_remaining_pct: ctxRemaining,
        context_tokens: ctxTokens,
      },
      cost: {
        ...(existing.cost || {}),
        session_usd: costUsd !== null ? costUsd : (existing.cost?.session_usd ?? null),
        session_tokens: ctxTokens,
      },
      active_agents: [{
        role: existing.active_agents?.[0]?.role || '',
        model: apiName || model,
        started_at: sessionStartedAt,
        session_id: sessionId,
      }],
    };

    writeHudAtomic(hudPath, hud);

    // Write per-session file for multi-session dashboard support
    try {
      fs.mkdirSync(sessionsDir, { recursive: true });
      // Detect the shell PID so VS Code extension can focus the right terminal.
      // Chain: VS Code terminal shell → claude → node(this script)
      // process.ppid = claude's PID; grandparent = shell's PID = terminal.processId
      let _shell_pid = 0;
      try {
        const { execSync: _ex } = require('child_process');
        const claudePid = process.ppid;
        const grandPid = parseInt(_ex(`ps -o ppid= -p ${claudePid} 2>/dev/null`).toString().trim(), 10);
        if (grandPid > 0) _shell_pid = grandPid;
      } catch {}
      writeHudAtomic(path.join(sessionsDir, sessionId + '.json'), {
        ...hud,
        _root: root,
        _session_id: sessionId,
        _last_updated: new Date().toISOString(),
        _shell_pid,
      });
      // Clean up session files older than 8 hours
      try {
        const cutoff = Date.now() - 8 * 60 * 60 * 1000;
        fs.readdirSync(sessionsDir).forEach(function(f) {
          if (!f.endsWith('.json')) return;
          try {
            const fp = path.join(sessionsDir, f);
            if (fs.statSync(fp).mtimeMs < cutoff) fs.unlinkSync(fp);
          } catch {}
        });
      } catch {}
    } catch {}

    // Terminal statusLine output
    const ADJ = ['bold','calm','swift','bright','sharp','keen','wild','quiet','brave','cool','warm','soft','fast','wise','pure','deft','lean','sage','red','blue','gold','jade','iron','amber','violet','azure','coral','frost','storm','sand','ember','cedar','steel','nova','oak','ivy','clay','moss','dawn','rust'];
    const NON = ['falcon','tiger','wolf','eagle','raven','fox','bear','hawk','lynx','crane','otter','pike','heron','wren','viper','bison','moose','ibis','kite','wasp','colt','finch','puma','cobra','gecko','quail','trout','mink','stork','stoat','dingo','snipe','marten','condor','osprey','ferret','oriole','magpie','jaguar','marlin'];
    let _h = 0;
    for (let _i = 0; _i < sessionId.length; _i++) _h = (Math.imul(_h, 31) + sessionId.charCodeAt(_i)) >>> 0;
    const nick = ADJ[_h % ADJ.length] + '-' + NON[(_h >>> 8) % NON.length];
    const parts = [];
    parts.push(nick);
    parts.push(model);
    if (costUsd !== null && costUsd > 0) parts.push(`$${costUsd.toFixed(3)}`);
    if (sessionStartedAt) parts.push(formatElapsed(sessionStartedAt));
    if (ctxRemaining !== null) {
      const used = Math.round(100 - ctxRemaining);
      const bar = '█'.repeat(Math.floor(used / 10)) + '░'.repeat(10 - Math.floor(used / 10));
      parts.push(`${bar} ${used}%`);
    }
    process.stdout.write(parts.join(' | '));
  } catch {
    process.exit(0);
  }
});
