use std::sync::Arc;

use cleaner_domain::SystemSnapshot;
use cleaner_service::AppService;
#[cfg(target_os = "macos")]
use objc2::MainThreadMarker;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSColor, NSForegroundColorAttributeName};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSMutableAttributedString, NSRange, NSString};
use tauri::menu::MenuBuilder;
use tauri::tray::TrayIcon;
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Emitter, Manager, Runtime};

use crate::lifecycle;

pub(crate) fn install(app: &mut App) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("show", "Open Mac Cleaner")
        .text("scan", "Scan now")
        .separator()
        .text("quit", "Quit Mac Cleaner")
        .build()?;

    TrayIconBuilder::with_id("mac-cleaner")
        .title("RAM …/… · SSD …/…")
        .tooltip("Mac Cleaner is reading available, free, and total memory and storage")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => lifecycle::show_main_window(app),
            "scan" => {
                lifecycle::show_main_window(app);
                let app = app.clone();
                let service = Arc::clone(app.state::<Arc<AppService>>().inner());
                tauri::async_runtime::spawn(async move {
                    match service.scan().await {
                        Ok(report) => {
                            let _ = app.emit("scan-complete", report);
                        }
                        Err(error) => {
                            let _ = app.emit("scan-error", error.to_string());
                        }
                    }
                });
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

pub(crate) fn update_status(app: &AppHandle, snapshot: &SystemSnapshot) {
    let Some(tray) = app.tray_by_id("mac-cleaner") else {
        return;
    };
    let memory = compact_pair(snapshot.memory.available_bytes, snapshot.memory.total_bytes);
    let disk = compact_pair(snapshot.disk.available_bytes, snapshot.disk.total_bytes);
    let memory_status = memory_status(snapshot.memory.available_bytes, snapshot.memory.total_bytes);
    let disk_status = disk_status(snapshot.disk.available_bytes, snapshot.disk.total_bytes);
    set_status_title(&tray, &memory, &disk, memory_status, disk_status);
    let _ = tray.set_tooltip(Some(format!(
        "Mac Cleaner — {} memory: {} RAM available out of {} ({} completely free); {} storage: {} available out of {}",
        memory_status.label,
        readable_bytes(snapshot.memory.available_bytes),
        readable_bytes(snapshot.memory.total_bytes),
        readable_bytes(snapshot.memory.free_bytes),
        disk_status.label,
        readable_bytes(snapshot.disk.available_bytes),
        readable_bytes(snapshot.disk.total_bytes),
    )));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StatusLevel {
    Unknown,
    Healthy,
    Low,
    Critical,
}

#[derive(Clone, Copy)]
struct MenuStatus {
    level: StatusLevel,
    label: &'static str,
}

fn memory_status(available: u64, total: u64) -> MenuStatus {
    ratio_status(available, total, 20, 10)
}

fn disk_status(available: u64, total: u64) -> MenuStatus {
    ratio_status(available, total, 15, 5)
}

fn ratio_status(available: u64, total: u64, healthy_percent: u64, low_percent: u64) -> MenuStatus {
    if total == 0 {
        return MenuStatus {
            level: StatusLevel::Unknown,
            label: "unknown",
        };
    }
    let percentage = available.saturating_mul(100) / total;
    if percentage >= healthy_percent {
        MenuStatus {
            level: StatusLevel::Healthy,
            label: "healthy",
        }
    } else if percentage >= low_percent {
        MenuStatus {
            level: StatusLevel::Low,
            label: "low",
        }
    } else {
        MenuStatus {
            level: StatusLevel::Critical,
            label: "critical",
        }
    }
}

fn set_status_title<R: Runtime>(
    tray: &TrayIcon<R>,
    memory: &str,
    disk: &str,
    memory_status: MenuStatus,
    disk_status: MenuStatus,
) {
    let memory_text = format!("RAM {memory}");
    let disk_text = format!("SSD {disk}");
    let plain_title = format!("{memory_text} · {disk_text}");

    #[cfg(target_os = "macos")]
    if set_macos_attributed_title(
        tray,
        plain_title.clone(),
        memory_text.encode_utf16().count(),
        disk_text.encode_utf16().count(),
        memory_status.level,
        disk_status.level,
    ) {
        return;
    }

    let _ = tray.set_title(Some(plain_title));
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn set_macos_attributed_title<R: Runtime>(
    tray: &TrayIcon<R>,
    title: String,
    memory_length: usize,
    disk_length: usize,
    memory_level: StatusLevel,
    disk_level: StatusLevel,
) -> bool {
    tray.with_inner_tray_icon(move |inner| {
        let Some(main_thread) = MainThreadMarker::new() else {
            return false;
        };
        let Some(status_item) = inner.ns_status_item() else {
            return false;
        };
        let Some(button) = status_item.button(main_thread) else {
            return false;
        };

        let attributed = NSMutableAttributedString::from_nsstring(&NSString::from_str(&title));
        let label_color = NSColor::labelColor();
        let memory_color = status_color(memory_level);
        let disk_color = status_color(disk_level);
        let disk_start = title.encode_utf16().count().saturating_sub(disk_length);

        // SAFETY: All ranges are calculated in UTF-16 code units for this exact NSString,
        // the attribute value is an NSColor as required by AppKit, and Tauri executes this
        // closure on the main thread while retaining the underlying NSStatusItem.
        unsafe {
            attributed.addAttribute_value_range(
                NSForegroundColorAttributeName,
                &label_color,
                NSRange::new(0, title.encode_utf16().count()),
            );
            attributed.addAttribute_value_range(
                NSForegroundColorAttributeName,
                &memory_color,
                NSRange::new(0, memory_length),
            );
            attributed.addAttribute_value_range(
                NSForegroundColorAttributeName,
                &disk_color,
                NSRange::new(disk_start, disk_length),
            );
            button.setAttributedTitle(&attributed);
        }
        true
    })
    .unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn status_color(level: StatusLevel) -> objc2::rc::Retained<NSColor> {
    match level {
        StatusLevel::Unknown => NSColor::secondaryLabelColor(),
        StatusLevel::Healthy => NSColor::systemGreenColor(),
        StatusLevel::Low => NSColor::systemOrangeColor(),
        StatusLevel::Critical => NSColor::systemRedColor(),
    }
}

fn compact_pair(available: u64, total: u64) -> String {
    format!("{}/{}G", gb_value(available), gb_value(total))
}

fn gb_value(bytes: u64) -> String {
    const GB: u64 = 1_000_000_000;
    if bytes >= 100 * GB {
        bytes
            .saturating_add(GB / 2)
            .checked_div(GB)
            .unwrap_or(0)
            .to_string()
    } else {
        let tenths = bytes.saturating_add(GB / 20) / (GB / 10);
        format!("{}.{:01}", tenths / 10, tenths % 10)
    }
}

fn readable_bytes(bytes: u64) -> String {
    format!("{} GB", gb_value(bytes))
}

#[cfg(test)]
mod tests {
    use super::{StatusLevel, compact_pair, disk_status, memory_status, readable_bytes};

    #[test]
    fn menu_bar_shows_available_out_of_total() {
        assert_eq!(compact_pair(6_000_000_000, 19_300_000_000), "6.0/19.3G");
        assert_eq!(compact_pair(2_300_000_000, 494_400_000_000), "2.3/494G");
        assert_eq!(readable_bytes(2_300_000_000), "2.3 GB");
    }

    #[test]
    fn menu_status_colors_use_resource_specific_thresholds() {
        assert_eq!(memory_status(3, 20).level, StatusLevel::Low);
        assert_eq!(memory_status(1, 20).level, StatusLevel::Critical);
        assert_eq!(memory_status(4, 20).level, StatusLevel::Healthy);
        assert_eq!(disk_status(4, 100).level, StatusLevel::Critical);
        assert_eq!(disk_status(10, 100).level, StatusLevel::Low);
        assert_eq!(disk_status(15, 100).level, StatusLevel::Healthy);
    }
}
