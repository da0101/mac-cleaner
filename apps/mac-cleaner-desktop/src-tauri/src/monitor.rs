use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use cleaner_domain::MemoryPressureLevel;
use cleaner_service::AppService;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

const ALERT_COOLDOWN: Duration = Duration::from_mins(15);

pub(crate) fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let service = Arc::clone(app.state::<Arc<AppService>>().inner());
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        let mut last_memory_alert: Option<(MemoryPressureLevel, Instant)> = None;
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            interval.tick().await;
            match service.system_snapshot().await {
                Ok(snapshot) => {
                    let now = Instant::now();
                    let pressure = snapshot.memory.pressure;
                    let should_alert = pressure != MemoryPressureLevel::Normal
                        && last_memory_alert.is_none_or(|(last_level, last_at)| {
                            pressure > last_level || now.duration_since(last_at) >= ALERT_COOLDOWN
                        });
                    if should_alert {
                        let (title, guidance) = match pressure {
                            MemoryPressureLevel::Critical => (
                                "Mac Cleaner: memory pressure is critical",
                                "Performance may be affected. Review the largest unprotected apps now; developer work will not be closed automatically.",
                            ),
                            MemoryPressureLevel::Warning => (
                                "Mac Cleaner: memory pressure is elevated",
                                "macOS is compressing and recycling memory. Review the largest unprotected apps if performance is slowing.",
                            ),
                            MemoryPressureLevel::Normal => {
                                unreachable!("normal pressure never alerts")
                            }
                        };
                        let _ = app
                            .notification()
                            .builder()
                            .title(title)
                            .body(format!(
                                "{} available, {} compressed, {} swap. {guidance}",
                                readable_bytes(snapshot.memory.available_bytes),
                                readable_bytes(snapshot.memory.compressed_bytes),
                                readable_bytes(snapshot.memory.swap_used_bytes),
                            ))
                            .show();
                        last_memory_alert = Some((pressure, now));
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

fn readable_bytes(bytes: u64) -> String {
    const TENTH_GB: u64 = 100_000_000;
    let tenths = bytes.saturating_add(TENTH_GB / 2) / TENTH_GB;
    format!("{}.{:01} GB", tenths / 10, tenths % 10)
}
