function resetTimers() {
  timers.forEach(clearInterval);
  timers = [];
  timers.push(setInterval(loadStatus, seconds('dashboard_status_interval_seconds', 10) * 1000));
  timers.push(setInterval(loadHistory, seconds('dashboard_history_interval_seconds', 60) * 1000));
  timers.push(setInterval(loadProcesses, seconds('dashboard_process_interval_seconds', 15) * 1000));
  timers.push(setInterval(loadAlerts, seconds('dashboard_alert_interval_seconds', 10) * 1000));
  if (shouldPollAI()) timers.push(setInterval(loadAIRecommendations, seconds('ai_recommendation_interval_seconds', 300) * 1000));
  if (shouldPollChromeTabs()) timers.push(setInterval(loadChromeTabs, seconds('chrome_tab_recommendation_interval_seconds', 300) * 1000));
  nextCleanTime = Date.now() + seconds('auto_clean_interval_seconds', 900) * 1000;
  nextRamPurge = Date.now() + seconds('ram_purge_interval_seconds', 300) * 1000;
}

// Auto-refresh
loadStatus().then(() => {
  resetTimers();
  scanNow();
  loadHistory();
  loadProcesses();
  loadAlerts();
  if (shouldPollAI()) loadAIRecommendations();
  if (shouldPollChromeTabs()) loadChromeTabs();
});
