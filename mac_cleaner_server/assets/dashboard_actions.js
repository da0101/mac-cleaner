async function loadStatus() {
  try {
    const data = await fetchJSON('/api/status');
    if (data.settings) {
      settings = data.settings;
      applySettings();
      fillSettingsForm();
    }
    autoEnabled = data.auto_clean_enabled;
    document.getElementById('autoStatus').textContent = autoEnabled ? 'ON' : 'OFF';
    document.getElementById('autoStatus').className = 'auto-badge ' + (autoEnabled ? 'on' : 'off');
    document.getElementById('btnToggle').textContent = autoEnabled ? 'Pause Auto-Clean' : 'Resume Auto-Clean';

    // System info
    const info = data.system;
    if (info) {
      const available = info.available_ram || info.free_ram || 0;
      const freeMB = Math.round((info.free_ram || 0) / 1024**2);
      const availableMB = Math.round(available / 1024**2);
      const totalRamGB = (info.total_ram / 1024**3).toFixed(1);
      const ramPct = info.total_ram ? Math.round((1 - available/info.total_ram)*100) : 0;
      // Show MB when under 1GB, GB otherwise
      const availableStr = availableMB < 1024 ? availableMB + ' MB' : (availableMB/1024).toFixed(1) + ' GB';
      document.getElementById('freeRam').textContent = availableStr;
      document.getElementById('freeRam').style.color = availableMB < 1024 ? 'var(--yellow)' : 'var(--green)';
      document.getElementById('ramBar').style.width = ramPct + '%';
      document.getElementById('ramBar').style.background = ramPct > 95 ? 'var(--red)' : ramPct > 85 ? 'var(--yellow)' : 'var(--green)';
      document.getElementById('ramSub').textContent = ramPct + '% pressure footprint of ' + totalRamGB + ' GB; raw free ' + freeMB + ' MB';
      // Breakdown
      const appGB = (info.ram_app / 1024**3).toFixed(1);
      const wiredGB = (info.ram_wired / 1024**3).toFixed(1);
      const compGB = (info.ram_compressed / 1024**3).toFixed(1);
      const cacheGB = (((info.ram_cached||0) + (info.ram_speculative||0) + (info.ram_purgeable||0)) / 1024**3).toFixed(1);
      document.getElementById('ramDetail').textContent = 'App: ' + appGB + 'G | Wired: ' + wiredGB + 'G | Compressed: ' + compGB + 'G | Reusable: ' + cacheGB + 'G';

      document.getElementById('diskFree').textContent = info.disk_available || '—';
      document.getElementById('diskBar').style.width = (info.disk_percent || 0) + '%';
      document.getElementById('diskBar').style.background = info.disk_percent > 90 ? 'var(--red)' : info.disk_percent > 75 ? 'var(--yellow)' : 'var(--blue)';
      document.getElementById('diskSub').textContent = (info.disk_percent||0) + '% used of ' + (info.disk_total||'—');
    }

    if (data.last_clean) {
      document.getElementById('lastClean').textContent = formatTime(data.last_clean);
      document.getElementById('lastCleanSub').textContent = formatDate(data.last_clean);
    }
  } catch(e) { console.error(e); }
}

async function scanNow() {
  const btn = document.getElementById('btnScan');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Scanning...';
  try {
    const data = await fetchJSON('/api/scan');
    renderGarbage(data.items);
    const cleanable = data.items.filter(i => i.cleanable);
    const total = cleanable.reduce((a,b) => a + b.size, 0);
    const reportOnly = data.items.length - cleanable.length;
    document.getElementById('garbageTotal').textContent = cleanable.length > 0 ? formatBytes(total) : '0 B';
    document.getElementById('garbageSub').textContent = cleanable.length + ' cleanable, ' + reportOnly + ' report-only';
    toast('Scan complete — ' + data.items.length + ' items found');
  } catch(e) { toast('Scan failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Scan Now';
}

function formatBytes(b) {
  if (b < 1024) return b + ' B';
  if (b < 1024**2) return (b/1024).toFixed(1) + ' KB';
  if (b < 1024**3) return (b/1024**2).toFixed(1) + ' MB';
  return (b/1024**3).toFixed(2) + ' GB';
}

function renderGarbage(items) {
  const body = document.getElementById('garbageBody');
  if (!items.length) { body.innerHTML = '<tr><td colspan="3" style="color:var(--dim)">System is clean!</td></tr>'; return; }
  body.innerHTML = items.map(i => `
    <tr>
      <td>${i.name}<div style="color:var(--dim);font-size:11px">${i.cleanable ? i.safety : 'report-only'} — ${i.reason || ''}</div></td>
      <td><span class="cat ${catClass(i.category)}">${i.category}</span></td>
      <td style="text-align:right"><span class="size ${sizeClass(i.size)}">${i.size_human}</span></td>
    </tr>
  `).join('');
}

async function cleanAll() {
  if (!confirm('Clean all scanner-approved cleanable items? This can include review-required caches/logs; report-only System Data stays protected.')) return;
  const btn = document.getElementById('btnClean');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Cleaning...';
  try {
    const data = await fetchJSON('/api/clean', {method:'POST'});
    toast('Cleaned ' + data.total_cleaned + ' (' + data.items_cleaned + ' items)');
    nextCleanTime = Date.now() + seconds('auto_clean_interval_seconds', 900) * 1000;
    loadStatus();
    scanNow();
    loadHistory();
  } catch(e) { toast('Clean failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Clean All Garbage';
}

async function dockerPrune() {
  if (!confirm('Run Docker prune? This can remove images, containers, build cache, and volumes.')) return;
  const btn = document.getElementById('btnDocker');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Pruning...';
  try {
    const data = await fetchJSON('/api/docker-prune', {method:'POST'});
    toast(data.success ? 'Docker pruned!' : 'Docker prune failed');
  } catch(e) { toast('Docker prune failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Docker Prune';
}

async function purgeRam() {
  if (!confirm('Purge inactive RAM now? Running apps stay open, but macOS may pause briefly.')) return;
  const btn = document.getElementById('btnRam');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Purging...';
  try {
    const data = await fetchJSON('/api/purge-ram', {method:'POST'});
    toast(data.success ? 'RAM purged!' : 'RAM purge needs sudo');
    loadStatus();
  } catch(e) { toast('Failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Purge RAM';
}

async function loadAIRecommendations() {
  try {
    const data = await fetchJSON('/api/ai-recommendations');
    renderAIRecommendations(data);
  } catch(e) {
    document.getElementById('aiSummary').textContent = 'AI recommendations unavailable.';
    document.getElementById('aiBody').innerHTML = '';
  }
}

async function refreshAI() {
  const btn = document.getElementById('btnAI');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Refreshing...';
  try {
    const data = await fetchJSON('/api/ai-recommendations', {method:'POST'});
    renderAIRecommendations(data);
    toast(data.provider === 'gemini' ? 'Gemini recommendations updated' : 'Local recommendations updated');
  } catch(e) { toast('AI refresh failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Refresh AI';
}

async function loadChromeTabs() {
  try {
    const data = await fetchJSON('/api/chrome-tab-recommendations');
    renderChromeTabRecommendations(data);
  } catch(e) {
    document.getElementById('chromeTabSummary').textContent = 'Chrome tab optimizer unavailable.';
    document.getElementById('chromeTabBody').innerHTML = '';
  }
}

async function refreshChromeTabs() {
  const btn = document.getElementById('btnChromeTabs');
  btn.disabled = true;
  btn.innerHTML = '<span class="spinner"></span> Checking...';
  try {
    const data = await fetchJSON('/api/chrome-tab-recommendations', {method:'POST'});
    renderChromeTabRecommendations(data);
    toast('Chrome tabs checked');
  } catch(e) { toast('Chrome tab check failed'); }
  btn.disabled = false;
  btn.innerHTML = 'Refresh Tabs';
}

function renderChromeTabRecommendations(data) {
  const body = document.getElementById('chromeTabBody');
  const summary = document.getElementById('chromeTabSummary');
  const status = document.getElementById('chromeTabStatus');
  const recs = data.recommendations || [];
  summary.textContent = data.summary || 'No Chrome tab summary available.';
  status.textContent = (data.provider || 'unknown') + (data.generated_at ? ' · ' + formatTime(data.generated_at) : '');
  if (!recs.length) {
    body.innerHTML = '<div style="color:var(--dim);font-size:13px">No tab cleanup recommendations.</div>';
    return;
  }
  body.innerHTML = recs.map(r => {
    const title = esc(r.title || 'Untitled');
    const domain = esc(r.domain || 'unknown');
    const reason = esc(r.reason || '');
    const url = esc(r.url || '');
    const priority = esc(r.priority || 'medium');
    const payload = encodeURIComponent(JSON.stringify({
      window_index: r.window_index,
      tab_index: r.tab_index,
      tab_id: r.tab_id || '',
      title: r.title || 'Untitled',
      domain: r.domain || '',
      url: r.url || ''
    }));
    return `<div class="ai-card">
      <div>
        <strong>${title}</strong>
        <div class="ai-meta">
          <span class="pill ${priority}">${priority}</span>
          <span class="pill review">review</span>
          <span style="color:var(--cyan);font-size:12px">${domain}</span>
        </div>
        <div style="font-size:12px;color:var(--dim)">${reason}</div>
        <div style="font-size:11px;color:var(--dim);margin-top:3px">${url}</div>
      </div>
      <div><button class="btn danger" onclick="closeChromeTab('${payload}')">Close Tab</button></div>
    </div>`;
  }).join('');
}

async function closeChromeTab(payload) {
  const data = JSON.parse(decodeURIComponent(payload));
  const title = data.title || 'this tab';
  if (!confirm('Close this Chrome tab?\n\n' + title + '\n\nThis cannot be undone by mac-cleaner.')) return;
  try {
    const result = await fetchJSON('/api/chrome-tabs/close', {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify(data)
    });
    toast(result.success ? 'Chrome tab closed' : (result.error || 'Tab close failed'));
    refreshChromeTabs();
  } catch(e) { toast('Tab close failed'); }
}

function renderAIRecommendations(data) {
  const body = document.getElementById('aiBody');
  const summary = document.getElementById('aiSummary');
  const status = document.getElementById('aiStatus');
  const recs = data.recommendations || [];
  summary.textContent = data.summary || 'No AI summary available.';
  status.textContent = (data.provider || 'unknown') + (data.generated_at ? ' · ' + formatTime(data.generated_at) : '');
  if (!recs.length) {
    body.innerHTML = '<div style="color:var(--dim);font-size:13px">No recommendations.</div>';
    return;
  }
  body.innerHTML = recs.map(r => {
    const action = esc(r.action || 'review');
    const label = esc(r.target_label || r.target_id || 'Recommendation');
    const reason = esc(r.reason || '');
    const savings = esc(r.expected_savings || '0 B');
    const priority = esc(r.priority || 'medium');
    const risk = esc(r.risk || 'review');
    let button = '<span style="color:var(--dim);font-size:12px">Review only</span>';
    if (r.action === 'purge_ram') {
      button = '<button class="btn" onclick="purgeRam()">Purge RAM</button>';
    } else if (r.action === 'clean_storage') {
      button = '<button class="btn" onclick="scanNow()">Review Scan</button>';
    }
    return `<div class="ai-card">
      <div>
        <strong>${label}</strong>
        <div class="ai-meta">
          <span class="pill ${priority}">${priority}</span>
          <span class="pill ${risk}">${risk}</span>
          <span style="color:var(--green);font-size:12px">${savings}</span>
          <span style="color:var(--dim);font-size:12px">${action}</span>
        </div>
        <div style="font-size:12px;color:var(--dim)">${reason}</div>
      </div>
      <div>${button}</div>
    </div>`;
  }).join('');
}
