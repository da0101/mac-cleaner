#!/usr/bin/env node
// workflow-parser.js — extract planned agents from a Workflow script
// Uses paren-counting to handle multi-line agent() calls correctly.
'use strict';

let raw = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', c => { raw += c; });
process.stdin.on('end', () => {
  let payload;
  try { payload = JSON.parse(raw); } catch { process.exit(0); }

  const script = payload.script || '';
  const agents = [];

  // ── Strategy 1: meta.agents array (preferred — explicit declaration) ────────
  // Matches: agents: [ {label:"...", model:"...", role:"...", skill:"..."}, ... ]
  const metaAgentsM = script.match(/agents\s*:\s*\[([\s\S]*?)\]/);
  if (metaAgentsM) {
    const block = metaAgentsM[1];
    const objRe = /\{([^}]+)\}/g;
    let om;
    while ((om = objRe.exec(block)) !== null) {
      const obj = om[1];
      const get = k => { const r = obj.match(new RegExp(k + '\\s*:\\s*[\'"`]([^\'"` ]+)[\'"`]')); return r ? r[1].trim() : ''; };
      const label = get('label');
      if (label) {
        const roleM = label.match(/role:\s*([^·|]+)/);
        const skillM = label.match(/skill:\s*([^·|]+)/);
        agents.push({
          label, model: get('model'), phase: get('phase'), agentType: get('agentType'),
          role: get('role') || (roleM ? roleM[1].trim() : ''),
          skill: get('skill') || (skillM ? skillM[1].trim() : ''),
          status: 'planned',
        });
      }
    }
  }

  // ── Strategy 2: parse agent() call sites via paren-counting ─────────────────
  if (!agents.length) {
    let i = 0;
    while (i < script.length) {
      // Find next agent( that isn't part of a longer identifier
      const idx = script.indexOf('agent(', i);
      if (idx === -1) break;
      // Make sure it's not e.g. "createAgent("
      const charBefore = idx > 0 ? script[idx - 1] : ' ';
      if (/\w/.test(charBefore)) { i = idx + 6; continue; }

      // Paren-count to find the full call body
      let depth = 0, j = idx + 5; // j points at '('
      let inStr = null;
      while (j < script.length) {
        const ch = script[j];
        if (inStr) {
          if (ch === '\\') { j++; } // skip escaped char
          else if (ch === inStr) inStr = null;
        } else if (ch === '"' || ch === "'" || ch === '`') {
          inStr = ch;
        } else if (ch === '(') {
          depth++;
        } else if (ch === ')') {
          depth--;
          if (depth === 0) break;
        }
        j++;
      }

      const callBody = script.slice(idx + 6, j); // contents of agent(...)

      // Extract label (handles single, double, template quotes, multi-line)
      const labelM = callBody.match(/label\s*:\s*['"`]([\s\S]*?)['"`]/);
      const modelM = callBody.match(/model\s*:\s*['"`]([^'"`\n, }]+)['"`]/);
      const phaseM = callBody.match(/phase\s*:\s*['"`]([^'"`\n, }]+)['"`]/);
      const agentTypeM = callBody.match(/agentType\s*:\s*['"`]([^'"`\n, }]+)['"`]/);

      // First arg (the prompt) as fallback label
      const firstArgM = callBody.match(/^\s*['"`]([\s\S]{1,80}?)['"`]/);
      const label = (labelM ? labelM[1] : firstArgM ? firstArgM[1] : '').trim().slice(0, 100);

      // Always push — use "Agent N" placeholder if no label found so total count is accurate
      const finalLabel = label || `Agent ${agents.length + 1}`;
      const roleM = finalLabel.match(/role:\s*([^·|]+)/);
      const skillM = finalLabel.match(/skill:\s*([^·|]+)/);
      agents.push({
        label: finalLabel,
        role: roleM ? roleM[1].trim() : '',
        skill: skillM ? skillM[1].trim() : '',
        model: modelM ? modelM[1] : '',
        phase: phaseM ? phaseM[1] : '',
        agentType: agentTypeM ? agentTypeM[1] : '',
        unlabeled: !label,
        status: 'planned',
      });

      i = j + 1;
    }
  }

  // ── Meta extraction ──────────────────────────────────────────────────────────
  const nameM = script.match(/name\s*:\s*['"`]([^'"`]+)['"`]/);
  const wfName = payload.name || (nameM ? nameM[1] : '') || 'workflow';

  const phases = [];
  const phasesM = script.match(/phases\s*:\s*\[([\s\S]*?)\]/);
  if (phasesM) {
    const tRe = /title\s*:\s*['"`]([^'"`]+)['"`]/g; let pm;
    while ((pm = tRe.exec(phasesM[1])) !== null) phases.push(pm[1]);
  }

  const result = {
    name: wfName,
    phases,
    agents,
    total: agents.length,
    started_at: new Date().toISOString(),
    status: 'running',
  };

  process.stdout.write(JSON.stringify(result, null, 2));
});
