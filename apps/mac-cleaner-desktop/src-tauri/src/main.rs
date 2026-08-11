mod commands;
mod lifecycle;
mod monitor;
mod tray;

use std::sync::Arc;

use anyhow::{Context, Result};
use cleaner_service::AppService;
use directories::{BaseDirs, ProjectDirs};
use tauri_plugin_autostart::MacosLauncher;

fn main() {
    if let Err(error) = run() {
        eprintln!("Mac Cleaner failed to start: {error:#}");
    }
}

fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("mac_cleaner=info")
        .with_target(false)
        .compact()
        .init();
    let service = Arc::new(open_service()?);
    let background = std::env::args().any(|argument| argument == "--background");

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            lifecycle::show_main_window(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .manage(service)
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::start_scan,
            commands::analyze_storage,
            commands::open_storage_owner,
            commands::confirm_uv_cache_prune,
            commands::confirm_simulator_recordings_cleanup,
            commands::latest_scan,
            commands::prepare_plan,
            commands::confirm_and_execute_plan,
            commands::recent_history,
            commands::latest_uv_cleanup_receipt,
            commands::latest_simulator_cleanup_receipt,
            commands::recent_tool_cleanup_receipts,
            commands::active_quarantine,
            commands::confirm_quarantine_action,
            commands::launch_at_login,
            commands::set_launch_at_login,
        ])
        .setup(move |app| {
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            app.set_dock_visibility(false);
            tray::install(app)?;
            monitor::start(app.handle().clone());
            if !background {
                lifecycle::show_main_window(app.handle());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .context("native application runtime stopped unexpectedly")?;
    Ok(())
}

fn open_service() -> Result<AppService> {
    let base = BaseDirs::new().context("macOS home directory is unavailable")?;
    let project = ProjectDirs::from("com", "da0101", "Mac Cleaner")
        .context("macOS application data directory is unavailable")?;
    AppService::open(
        base.home_dir().to_path_buf(),
        project.data_dir().to_path_buf(),
    )
    .context("cannot initialize the Rust application service")
}
