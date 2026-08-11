use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use cleaner_service::AppService;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

const LOW_MEMORY_BYTES: u64 = 1_000_000_000;
const ALERT_COOLDOWN: Duration = Duration::from_mins(15);

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let service = Arc::clone(app.state::<Arc<AppService>>().inner());
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        let mut last_memory_alert: Option<Instant> = None;
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            match service.system_snapshot().await {
                Ok(snapshot) => {
                    let now = Instant::now();
                    if snapshot.memory.available_bytes < LOW_MEMORY_BYTES
                        && last_memory_alert
                            .is_none_or(|last| now.duration_since(last) >= ALERT_COOLDOWN)
                    {
                        let _ = app
                            .notification()
                            .builder()
                            .title("Mac Cleaner: memory is critically low")
                            .body("Less than 1 GB is available. Review the top processes; Mac Cleaner will not close apps automatically.")
                            .show();
                        last_memory_alert = Some(now);
                    }
                    crate::tray::update_status(&app, &snapshot);
                    let _ = app.emit("system-snapshot", snapshot);
                }
                Err(error) => {
                    let _ = app.emit("monitor-error", error.to_string());
                }
            }
        }
    });
}
