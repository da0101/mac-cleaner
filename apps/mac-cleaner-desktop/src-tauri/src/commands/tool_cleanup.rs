use std::sync::Arc;

use cleaner_domain::{ToolCleanupOperation, ToolCleanupReceipt};
use cleaner_service::AppService;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use super::helpers::{confirm_native, display_error};

#[tauri::command]
pub(crate) async fn confirm_uv_cache_prune(
    app: AppHandle,
    service: State<'_, Arc<AppService>>,
) -> Result<ToolCleanupReceipt, String> {
    let plan = service
        .prepare_uv_cache_prune()
        .await
        .map_err(display_error)?;
    let ToolCleanupOperation::Command {
        executable_path,
        arguments,
    } = &plan.operation
    else {
        return Err("uv cache cleanup plan is invalid. Nothing was changed.".to_owned());
    };
    let message = format!(
        "Owning tool: uv\nExact cache:\n{}\n\nCommand:\n{} {}\n\nMeasured cache: {}\n\nuv will remove only unreachable cache objects and will keep its built-in in-use checks. Python environments, projects, lockfiles, and installed packages are outside this target. Future installs may need to download packages again.",
        plan.target_path.display(),
        executable_path.display(),
        arguments.join(" "),
        human_bytes(plan.bytes_before),
    );
    let confirmed = confirm_native(
        &app,
        "Confirm uv cache prune",
        message,
        MessageDialogKind::Warning,
        "Prune unused uv cache",
    )
    .await?;
    if !confirmed {
        return Err("uv cache cleanup cancelled. Nothing was changed.".to_owned());
    }
    service
        .approve_tool_cleanup(plan.id)
        .map_err(display_error)?;
    service
        .execute_tool_cleanup(plan.id)
        .await
        .map_err(display_error)
}

#[tauri::command]
pub(crate) async fn confirm_simulator_recordings_cleanup(
    app: AppHandle,
    service: State<'_, Arc<AppService>>,
) -> Result<ToolCleanupReceipt, String> {
    let plan = service
        .prepare_simulator_recordings_cleanup()
        .await
        .map_err(display_error)?;
    let ToolCleanupOperation::SimulatorAttachments {
        device_name,
        device_was_booted,
        files,
        ..
    } = &plan.operation
    else {
        return Err("Simulator cleanup plan is invalid. Nothing was changed.".to_owned());
    };
    let shutdown = if *device_was_booted {
        "This simulator is currently booted. Mac Cleaner will shut down only this simulator before deleting the recordings."
    } else {
        "The simulator is already shut down."
    };
    let message = format!(
        "Device: {device_name}\nExact attachment directory:\n{}\n\nVerified old QuickTime files: {}\nMeasured reclaimable space: {}\n\n{shutdown}\n\nOnly direct video files older than 72 hours are included. Simulator apps, app data, source code, test results outside this directory, and the simulator device itself are not touched. If xcodebuild or XCTest is active, cleanup will stop.\n\nThis deletion is permanent; rerun the tests to recreate recordings.",
        plan.target_path.display(),
        files.len(),
        human_bytes(plan.bytes_before),
    );
    let confirmed = confirm_native(
        &app,
        "Confirm simulator recording cleanup",
        message,
        MessageDialogKind::Warning,
        "Remove old test recordings",
    )
    .await?;
    if !confirmed {
        return Err("Simulator recording cleanup cancelled. Nothing was changed.".to_owned());
    }
    service
        .approve_tool_cleanup(plan.id)
        .map_err(display_error)?;
    service
        .execute_tool_cleanup(plan.id)
        .await
        .map_err(display_error)
}

#[tauri::command]
pub(crate) fn latest_uv_cleanup_receipt(
    service: State<'_, Arc<AppService>>,
) -> Result<Option<ToolCleanupReceipt>, String> {
    service.latest_uv_cleanup_receipt().map_err(display_error)
}

#[tauri::command]
pub(crate) fn latest_simulator_cleanup_receipt(
    service: State<'_, Arc<AppService>>,
) -> Result<Option<ToolCleanupReceipt>, String> {
    service
        .latest_simulator_cleanup_receipt()
        .map_err(display_error)
}

#[tauri::command]
pub(crate) fn recent_tool_cleanup_receipts(
    service: State<'_, Arc<AppService>>,
    limit: u32,
) -> Result<Vec<ToolCleanupReceipt>, String> {
    service
        .recent_tool_cleanup_receipts(limit)
        .map_err(display_error)
}

fn human_bytes(bytes: u64) -> String {
    let tenths = bytes.saturating_add(50_000_000) / 100_000_000;
    format!("{}.{:01} GB", tenths / 10, tenths % 10)
}
