let nextCleanTime = Date.now() + 15 * 60 * 1000;
let nextRamPurge = Date.now() + 3 * 60 * 1000;
let autoEnabled = true;
let settings = {};
let timers = [];

function toast(msg, dur=3000) {
  const t = document.getElementById('toast');
  t.textContent = msg;
  t.classList.add('show');
  setTimeout(() => t.classList.remove('show'), dur);
}

function sizeClass(bytes) {
  if (bytes > 1024**3) return 'large';
  if (bytes > 100*1024**2) return 'medium';
  return 'small';
}

function catClass(cat) {
  return 'cat-' + (cat || 'cache');
}

function formatTime(iso) {
  if (!iso) return '—';
  const d = new Date(iso);
  return d.toLocaleTimeString([], {hour:'2-digit',minute:'2-digit'});
}

function formatDate(iso) {
  if (!iso) return '';
  const d = new Date(iso);
  const now = new Date();
  if (d.toDateString() === now.toDateString()) return 'Today ' + formatTime(iso);
  return d.toLocaleDateString() + ' ' + formatTime(iso);
}

async function fetchJSON(url, opts) {
  const r = await fetch(url, opts);
  return r.json();
}

function esc(v) {
  return String(v ?? '').replace(/[&<>"']/g, ch => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]));
}

function seconds(name, fallback) {
  return Math.max(1, Number(settings[name] || fallback));
}

function applySettings() {
  const theme = settings.theme || 'dark';
  const systemLight = window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches;
  document.body.classList.toggle('light', theme === 'light' || (theme === 'system' && systemLight));
  const themeBtn = document.getElementById('themeToggle');
  if (themeBtn) themeBtn.textContent = document.body.classList.contains('light') ? '☾' : '☼';
  setSectionVisible('aiSection', settings.show_ai_recommendations !== false);
  setSectionVisible('chromeTabsSection', settings.show_chrome_tab_optimizer !== false);
  setSectionVisible('garbageSection', settings.show_garbage_breakdown !== false);
  setSectionVisible('ramSection', settings.show_ram_breakdown !== false);
  setSectionVisible('historySection', settings.show_cleanup_history !== false);
  setSectionVisible('suggestionsSection', settings.show_rule_recommendations !== false);
}

function setSectionVisible(id, visible) {
  const el = document.getElementById(id);
  if (el) el.style.display = visible ? 'block' : 'none';
}

function shouldPollAI() {
  return settings.show_ai_recommendations !== false;
}

function shouldPollChromeTabs() {
  return settings.show_chrome_tab_optimizer !== false;
}

function fillSettingsForm() {
  const map = [
    ['setTheme','theme'], ['setAiInterval','ai_recommendation_interval_seconds'],
    ['setChromeInterval','chrome_tab_recommendation_interval_seconds'], ['setRamCooldown','ai_auto_optimize_cooldown_seconds'],
    ['setTargetRam','ai_target_available_ram_mb'], ['setAutoClean','auto_clean_interval_seconds'],
    ['setRamPurge','ram_purge_interval_seconds']
  ];
  map.forEach(([id,key]) => { const el = document.getElementById(id); if (el && document.activeElement !== el) el.value = settings[key]; });
  [['showAI','show_ai_recommendations'], ['showChrome','show_chrome_tab_optimizer'], ['showGarbage','show_garbage_breakdown'], ['showRam','show_ram_breakdown'], ['showRules','show_rule_recommendations'], ['showHistory','show_cleanup_history']]
    .forEach(([id,key]) => { const el = document.getElementById(id); if (el) el.checked = settings[key] !== false; });
}

function readSettingsForm() {
  return {
    theme: document.getElementById('setTheme').value,
    ai_recommendation_interval_seconds: Number(document.getElementById('setAiInterval').value),
    chrome_tab_recommendation_interval_seconds: Number(document.getElementById('setChromeInterval').value),
    ai_auto_optimize_cooldown_seconds: Number(document.getElementById('setRamCooldown').value),
    ai_target_available_ram_mb: Number(document.getElementById('setTargetRam').value),
    auto_clean_interval_seconds: Number(document.getElementById('setAutoClean').value),
    ram_purge_interval_seconds: Number(document.getElementById('setRamPurge').value),
    show_ai_recommendations: document.getElementById('showAI').checked,
    show_chrome_tab_optimizer: document.getElementById('showChrome').checked,
    show_garbage_breakdown: document.getElementById('showGarbage').checked,
    show_ram_breakdown: document.getElementById('showRam').checked,
    show_rule_recommendations: document.getElementById('showRules').checked,
    show_cleanup_history: document.getElementById('showHistory').checked,
  };
}

async function saveSettings() {
  try {
    settings = await fetchJSON('/api/settings', {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify(readSettingsForm())
    });
    applySettings();
    fillSettingsForm();
    resetTimers();
    closeSettings();
    toast('Settings saved');
  } catch(e) { toast('Settings save failed'); }
}

function openSettings() {
  fillSettingsForm();
  const modal = document.getElementById('settingsModal');
  modal.classList.add('open');
  setTimeout(() => document.getElementById('setTheme').focus(), 0);
}

function closeSettings() {
  document.getElementById('settingsModal').classList.remove('open');
}

async function toggleTheme() {
  const isLight = document.body.classList.contains('light');
  settings.theme = isLight ? 'dark' : 'light';
  applySettings();
  fillSettingsForm();
  try {
    settings = await fetchJSON('/api/settings', {
      method:'POST',
      headers:{'Content-Type':'application/json'},
      body: JSON.stringify(settings)
    });
    applySettings();
    fillSettingsForm();
  } catch(e) { toast('Theme save failed'); }
}

document.addEventListener('keydown', e => {
  if (e.key === 'Escape') closeSettings();
});
