async function toggleAuto() {
  try {
    const data = await fetchJSON('/api/auto-clean', {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify({enabled: !autoEnabled})
    });
    autoEnabled = data.enabled;
    syncCountdowns(data);
    document.getElementById('autoStatus').textContent = autoEnabled ? 'ON' : 'OFF';
    document.getElementById('autoStatus').className = 'auto-badge ' + (autoEnabled ? 'on' : 'off');
    document.getElementById('btnToggle').textContent = autoEnabled ? 'Pause Auto-Clean' : 'Enable Auto-Clean';
    toast('Auto-clean ' + (autoEnabled ? 'enabled' : 'paused'));
  } catch(e) {}
}

async function loadHistory() {
  try {
    const data = await fetchJSON('/api/history');
    const el = document.getElementById('historyBody');
    if (!data.length) { el.innerHTML = '<div class="history-item" style="color:var(--dim)">No cleanups yet</div>'; return; }
    el.innerHTML = data.slice(0,20).map(h => `
      <div class="history-item">
        <span class="history-time">${formatDate(h.timestamp)}</span>
        &nbsp;&middot;&nbsp;
        Cleaned <strong>${h.total_cleaned}</strong> from ${h.items_cleaned} items
      </div>
    `).join('');
  } catch(e) {}
}

// Countdown timer
setInterval(() => {
  if (!autoEnabled) { document.getElementById('countdown').textContent = ''; return; }
  const remaining = Math.max(0, Math.round((nextCleanTime - Date.now()) / 1000));
  const m = Math.floor(remaining / 60);
  const s = remaining % 60;
  const ramRem = Math.max(0, Math.round((nextRamPurge - Date.now()) / 1000));
  const rm = Math.floor(ramRem / 60);
  const rs = ramRem % 60;
  document.getElementById('countdown').textContent = 'RAM purge: ' + rm + ':' + String(rs).padStart(2,'0') + ' | Garbage clean: ' + m + ':' + String(s).padStart(2,'0');
}, 1000);

const CAT_COLORS = {
  system: '#8b949e', browser: '#f85149', ide: '#d29922', dev: '#58a6ff',
  docker: '#bc8cff', app: '#39d2c0', other: '#6e7681'
};
const CAT_LABELS = {
  system: 'System', browser: 'Browser', ide: 'IDE', dev: 'Dev Tool',
  docker: 'Docker', app: 'App', other: 'Other'
};

async function loadProcesses() {
  try {
    const data = await fetchJSON('/api/top-processes');
    const procs = data.processes || [];
    const suggestions = data.suggestions || [];
    const body = document.getElementById('processBody');

    if (!procs.length) { body.innerHTML = '<tr><td colspan="5" style="color:var(--dim)">No data</td></tr>'; return; }

    // Summary bar
    const totalRam = data.total_ram || 1;
    const barEl = document.getElementById('ramSummaryBar');
    const catTotals = {};
    procs.forEach(p => { catTotals[p.category] = (catTotals[p.category]||0) + p.mem_bytes; });
    barEl.innerHTML = Object.entries(catTotals).map(([cat, bytes]) => {
      const pct = Math.max(2, (bytes/totalRam)*100);
      return `<div style="width:${pct}%;background:${CAT_COLORS[cat]||'#444'};border-radius:3px" title="${CAT_LABELS[cat]||cat}: ${formatBytes(bytes)}"></div>`;
    }).join('');

    const summaryEl = document.getElementById('ramSummaryText');
    summaryEl.textContent = `Available: ${data.available_ram_human || data.free_ram_human} | Raw free: ${data.free_ram_human} | Closeable: ${data.closeable_ram_human} | System (locked): ${data.system_ram_human}`;

    // Process table
    body.innerHTML = procs.map(p => {
      const catColor = CAT_COLORS[p.category] || '#6e7681';
      const catLabel = CAT_LABELS[p.category] || p.category;
      let statusBadge;
      if (!p.can_close) {
        statusBadge = '<span style="color:var(--dim);font-size:11px">LOCKED</span>';
      } else if (p.mem_bytes > 300*1024*1024) {
        statusBadge = '<span style="color:var(--red);font-weight:600;font-size:11px">CLOSE</span>';
      } else if (p.mem_bytes > 50*1024*1024) {
        statusBadge = '<span style="color:var(--yellow);font-size:11px">CHECK</span>';
      } else {
        statusBadge = '<span style="color:var(--dim);font-size:11px">ok</span>';
      }
      const countStr = p.count > 1 ? ' <span style="color:var(--dim);font-size:11px">x' + p.count + '</span>' : '';
      const memColor = p.mem_bytes > 500*1024*1024 ? 'var(--red)' : p.mem_bytes > 200*1024*1024 ? 'var(--yellow)' : 'var(--text)';
      return `<tr>
        <td><strong>${p.label}</strong>${countStr}</td>
        <td><span class="cat" style="background:${catColor}22;color:${catColor}">${catLabel}</span></td>
        <td>${statusBadge}</td>
        <td style="text-align:right"><span style="color:${memColor};font-weight:600;font-variant-numeric:tabular-nums">${p.mem}</span></td>
        <td style="font-size:12px;color:var(--dim)">${p.suggestion}</td>
      </tr>`;
    }).join('');

    // Suggestions
    const sugSection = document.getElementById('suggestionsSection');
    const sugBody = document.getElementById('suggestionsBody');
    if (suggestions.length && settings.show_rule_recommendations !== false) {
      sugSection.style.display = 'block';
      sugBody.innerHTML = suggestions.map(s => {
        const icon = s.priority === 'high' ? '🔴' : s.priority === 'medium' ? '🟡' : '🟢';
        return `<div style="padding:8px 0;border-bottom:1px solid var(--border);font-size:13px">
          ${icon} <strong>${s.action}</strong> — saves <span style="color:var(--green);font-weight:600">${s.saves}</span>
          <span style="color:var(--dim);margin-left:8px">${s.reason}</span>
        </div>`;
      }).join('');
    } else {
      sugSection.style.display = 'none';
    }
  } catch(e) { console.error(e); }
}

async function loadAlerts() {
  try {
    const alerts = await fetchJSON('/api/alerts');
    const banner = document.getElementById('alertBanner');
    const body = document.getElementById('alertBody');
    const critical = alerts.filter(a => a.level === 'critical');
    const warnings = alerts.filter(a => a.level === 'warning');

    if (critical.length > 0) {
      banner.style.display = 'block';
      banner.style.background = '#3b1520';
      banner.style.borderColor = 'var(--red)';
      document.getElementById('alertTitle').textContent = '⚠ MEMORY LEAK — Action Required';
      document.getElementById('alertTitle').style.color = 'var(--red)';
      body.innerHTML = critical.map(a =>
        `<div style="margin:6px 0;display:flex;justify-content:space-between;align-items:center">
          <span><strong style="color:var(--red)">${a.label}</strong> is using <strong>${a.mem_gb} GB</strong> — ${a.message}</span>
          <span style="color:var(--dim);font-size:12px">Save work, then restart the app</span>
        </div>`
      ).join('') + (warnings.length ? '<div style="margin-top:8px;color:var(--yellow);font-size:12px">+ ' + warnings.length + ' warnings (2-4 GB range)</div>' : '');
    } else if (warnings.length > 0) {
      banner.style.display = 'block';
      banner.style.background = '#2a1f00';
      banner.style.borderColor = 'var(--yellow)';
      document.getElementById('alertTitle').textContent = '⚡ High Memory Usage';
      document.getElementById('alertTitle').style.color = 'var(--yellow)';
      body.innerHTML = warnings.map(a =>
        `<div style="margin:4px 0"><strong style="color:var(--yellow)">${a.label}</strong> — ${a.mem_gb} GB</div>`
      ).join('');
    } else {
      banner.style.display = 'none';
    }
  } catch(e) {}
}
